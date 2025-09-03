use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FetchOrdersFilter {
    All,
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOrdersResponse {
    #[serde(alias = "orderHash")]
    pub order_hash: String,
    pub owner: String,
    #[serde(alias = "blockTimestamp")]
    pub block_timestamp: u64,
    #[serde(alias = "blockNumber")]
    pub block_number: u64,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    #[serde(alias = "tradeCount")]
    pub trade_count: u64,
    pub status: String,
}

impl LocalDbQuery {
    pub async fn fetch_orders(
        db_callback: &js_sys::Function,
        filter: FetchOrdersFilter,
    ) -> Result<Vec<FetchOrdersResponse>, LocalDbQueryError> {
        let filter_str = match filter {
            FetchOrdersFilter::All => "all",
            FetchOrdersFilter::Active => "active",
            FetchOrdersFilter::Inactive => "inactive",
        };

        let sql = QUERY.replace("'?filter'", &format!("'{}'", filter_str));

        LocalDbQuery::execute_query_json::<Vec<FetchOrdersResponse>>(db_callback, &sql).await
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
                FetchOrdersResponse {
                    order_hash:
                        "0xabc0000000000000000000000000000000000000000000000000000000000001".into(),
                    owner: "0x1111111111111111111111111111111111111111".into(),
                    block_timestamp: 1000,
                    block_number: 123,
                    inputs: Some("1:0xaaa,2:0xbbb".into()),
                    outputs: Some("3:0xccc".into()),
                    trade_count: 2,
                    status: "active".into(),
                },
                FetchOrdersResponse {
                    order_hash:
                        "0xabc0000000000000000000000000000000000000000000000000000000000002".into(),
                    owner: "0x2222222222222222222222222222222222222222".into(),
                    block_timestamp: 2000,
                    block_number: 456,
                    inputs: None,
                    outputs: None,
                    trade_count: 0,
                    status: "inactive".into(),
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
            assert_eq!(data[0].inputs, orders[0].inputs);
            assert_eq!(data[0].outputs, orders[0].outputs);
            assert_eq!(data[0].trade_count, orders[0].trade_count);
            assert_eq!(data[0].status, orders[0].status);
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
