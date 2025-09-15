use super::*;
use crate::raindex_client::local_db::bool_from_int_or_bool;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FetchOrdersFilter {
    All,
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDbOrder {
    #[serde(alias = "orderHash")]
    pub order_hash: String,
    pub owner: String,
    #[serde(alias = "blockTimestamp")]
    pub block_timestamp: u64,
    #[serde(alias = "blockNumber")]
    pub block_number: u64,
    #[serde(alias = "orderbookAddress")]
    pub orderbook_address: String,
    #[serde(alias = "orderBytes")]
    pub order_bytes: String,
    #[serde(alias = "transactionHash")]
    pub transaction_hash: String,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    #[serde(alias = "tradeCount")]
    pub trade_count: u64,
    #[serde(deserialize_with = "bool_from_int_or_bool")]
    pub active: bool,
    pub meta: Option<String>,
}

impl LocalDbQuery {
    pub async fn fetch_orders(
        db_callback: &js_sys::Function,
        filter: FetchOrdersFilter,
    ) -> Result<Vec<LocalDbOrder>, LocalDbQueryError> {
        let filter_str = match filter {
            FetchOrdersFilter::All => "all",
            FetchOrdersFilter::Active => "active",
            FetchOrdersFilter::Inactive => "inactive",
        };

        let sql = QUERY.replace("'?filter'", &format!("'{}'", filter_str));

        LocalDbQuery::execute_query_json::<Vec<LocalDbOrder>>(db_callback, &sql).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::{
            create_sql_capturing_callback, create_success_callback,
        };
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_fetch_orders_parses_data() {
            let orders = vec![
                LocalDbOrder {
                    order_hash:
                        "0xabc0000000000000000000000000000000000000000000000000000000000001".into(),
                    owner: "0x1111111111111111111111111111111111111111".into(),
                    block_timestamp: 1000,
                    block_number: 123,
                    orderbook_address: "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB".into(),
                    order_bytes: "0xdeadbeef".into(),
                    transaction_hash:
                        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    inputs: Some("1:0xaaa,2:0xbbb".into()),
                    outputs: Some("3:0xccc".into()),
                    trade_count: 2,
                    active: true,
                    meta: Some("0x010203".into()),
                },
                LocalDbOrder {
                    order_hash:
                        "0xabc0000000000000000000000000000000000000000000000000000000000002".into(),
                    owner: "0x2222222222222222222222222222222222222222".into(),
                    block_timestamp: 2000,
                    block_number: 456,
                    orderbook_address: "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB".into(),
                    order_bytes: "0x00".into(),
                    transaction_hash:
                        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                    inputs: None,
                    outputs: None,
                    trade_count: 0,
                    active: false,
                    meta: None,
                },
            ];
            let json_data = serde_json::to_string(&orders).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_orders(&callback, FetchOrdersFilter::All).await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].order_hash, orders[0].order_hash);
            assert_eq!(data[0].owner, orders[0].owner);
            assert_eq!(data[0].block_timestamp, orders[0].block_timestamp);
            assert_eq!(data[0].block_number, orders[0].block_number);
            assert_eq!(data[0].orderbook_address, orders[0].orderbook_address);
            assert_eq!(data[0].order_bytes, orders[0].order_bytes);
            assert_eq!(data[0].transaction_hash, orders[0].transaction_hash);
            assert_eq!(data[0].inputs, orders[0].inputs);
            assert_eq!(data[0].outputs, orders[0].outputs);
            assert_eq!(data[0].trade_count, orders[0].trade_count);
            assert_eq!(data[0].active, orders[0].active);
            assert_eq!(data[0].meta, orders[0].meta);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_empty() {
            let callback = create_success_callback("[]");
            let result = LocalDbQuery::fetch_orders(&callback, FetchOrdersFilter::All).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_filter_replacement_all() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ = LocalDbQuery::fetch_orders(&callback, FetchOrdersFilter::All).await;

            let sql = captured_sql.borrow();
            assert!(sql.contains("'all'"), "SQL should contain 'all': {}", *sql);
            assert!(
                !sql.contains("?filter"),
                "SQL should not contain placeholder ?filter: {}",
                *sql
            );
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_filter_replacement_active() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ = LocalDbQuery::fetch_orders(&callback, FetchOrdersFilter::Active).await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains("'active'"),
                "SQL should contain 'active': {}",
                *sql
            );
            assert!(
                !sql.contains("?filter"),
                "SQL should not contain placeholder ?filter: {}",
                *sql
            );
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_filter_replacement_inactive() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ = LocalDbQuery::fetch_orders(&callback, FetchOrdersFilter::Inactive).await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains("'inactive'"),
                "SQL should contain 'inactive': {}",
                *sql
            );
            assert!(
                !sql.contains("?filter"),
                "SQL should not contain placeholder ?filter: {}",
                *sql
            );
        }
    }
}
