use super::*;
use crate::raindex_client::local_db::bool_from_int_or_bool;
use crate::raindex_client::orders::GetOrdersFilters;
use alloy::primitives::Bytes;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FetchOrdersActiveFilter {
    All,
    Active,
    Inactive,
}

#[derive(Debug, Clone)]
pub struct FetchOrdersArgs {
    pub filter: FetchOrdersActiveFilter,
    pub owners: Vec<Address>,
    pub order_hash: Option<Bytes>,
    pub tokens: Vec<Address>,
}

impl Default for FetchOrdersArgs {
    fn default() -> Self {
        Self {
            filter: FetchOrdersActiveFilter::All,
            owners: Vec::new(),
            order_hash: None,
            tokens: Vec::new(),
        }
    }
}

impl From<GetOrdersFilters> for FetchOrdersArgs {
    fn from(filters: GetOrdersFilters) -> Self {
        let filter = match filters.active {
            Some(true) => FetchOrdersActiveFilter::Active,
            Some(false) => FetchOrdersActiveFilter::Inactive,
            None => FetchOrdersActiveFilter::All,
        };

        FetchOrdersArgs {
            filter,
            owners: filters.owners,
            order_hash: filters.order_hash,
            tokens: filters.tokens.unwrap_or_default(),
        }
    }
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
    #[serde(alias = "interpreterAddress")]
    pub interpreter_address: String,
    #[serde(alias = "storeAddress")]
    pub store_address: String,
    #[serde(alias = "interpreterBytecode")]
    pub interpreter_bytecode: String,
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
        args: FetchOrdersArgs,
    ) -> Result<Vec<LocalDbOrder>, LocalDbQueryError> {
        let FetchOrdersArgs {
            filter,
            owners,
            order_hash,
            tokens,
        } = args;

        let filter_str = match filter {
            FetchOrdersActiveFilter::All => "all",
            FetchOrdersActiveFilter::Active => "active",
            FetchOrdersActiveFilter::Inactive => "inactive",
        };

        let owner_values: Vec<String> = owners
            .into_iter()
            .map(|owner| format!("'{owner:#x}'"))
            .collect();
        let filter_owners = if owner_values.is_empty() {
            String::new()
        } else {
            format!(
                "\nAND lower(l.order_owner) IN ({})\n",
                owner_values.join(", ")
            )
        };

        let filter_order_hash = order_hash
            .and_then(|hash| {
                if hash.is_empty() {
                    return None;
                }

                Some(format!(
                    "\nAND lower(COALESCE(la.order_hash, l.order_hash)) = lower('{}')\n",
                    hash
                ))
            })
            .unwrap_or_default();

        let token_values: Vec<String> = tokens
            .into_iter()
            .map(|token| format!("'{token:#x}'"))
            .collect();
        let filter_tokens = if token_values.is_empty() {
            String::new()
        } else {
            format!(
                "\nAND EXISTS (\n    SELECT 1 FROM order_ios io2\n    WHERE io2.transaction_hash = la.transaction_hash\n      AND io2.log_index = la.log_index\n      AND lower(io2.token) IN ({})\n)\n",
                token_values.join(", ")
            )
        };

        let sql = QUERY
            .replace("'?filter_active'", &format!("'{}'", filter_str))
            .replace("?filter_owners", &filter_owners)
            .replace("?filter_order_hash", &filter_order_hash)
            .replace("?filter_tokens", &filter_tokens);

        LocalDbQuery::execute_query_json::<Vec<LocalDbOrder>>(db_callback, &sql).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod host_tests {
        use super::*;
        use alloy::primitives::{Address, Bytes};
        use std::str::FromStr;

        #[test]
        fn test_fetch_orders_args_from_filters_active_true() {
            let filters = GetOrdersFilters {
                owners: vec![
                    Address::from_str("0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap(),
                ],
                active: Some(true),
                order_hash: Some(
                    Bytes::from_str(
                        "0xabc0000000000000000000000000000000000000000000000000000000000001",
                    )
                    .unwrap(),
                ),
                tokens: Some(vec![Address::from_str(
                    "0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                )
                .unwrap()]),
            };

            let args = FetchOrdersArgs::from(filters);

            assert!(matches!(args.filter, FetchOrdersActiveFilter::Active));
            assert_eq!(
                args.owners,
                vec![Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap()]
            );
            assert_eq!(
                args.order_hash,
                Some(
                    Bytes::from_str(
                        "0xabc0000000000000000000000000000000000000000000000000000000000001"
                    )
                    .unwrap()
                )
            );
            assert_eq!(
                args.tokens,
                vec![Address::from_str("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap()]
            );
        }

        #[test]
        fn test_fetch_orders_args_from_filters_defaults() {
            let filters = GetOrdersFilters {
                owners: vec![],
                active: None,
                order_hash: None,
                tokens: None,
            };

            let args = FetchOrdersArgs::from(filters);

            assert!(matches!(args.filter, FetchOrdersActiveFilter::All));
            assert!(args.owners.is_empty());
            assert!(args.order_hash.is_none());
            assert!(args.tokens.is_empty());
        }
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::{
            create_sql_capturing_callback, create_success_callback,
        };
        use alloy::primitives::{Address, Bytes};
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::str::FromStr;
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
                    interpreter_address: "0x3333333333333333333333333333333333333333".into(),
                    store_address: "0x4444444444444444444444444444444444444444".into(),
                    interpreter_bytecode: "0x01020304".into(),
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
                    interpreter_address: "0x5555555555555555555555555555555555555555".into(),
                    store_address: "0x6666666666666666666666666666666666666666".into(),
                    interpreter_bytecode: "0x05060708".into(),
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

            let result = LocalDbQuery::fetch_orders(&callback, FetchOrdersArgs::default()).await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].order_hash, orders[0].order_hash);
            assert_eq!(data[0].owner, orders[0].owner);
            assert_eq!(data[0].block_timestamp, orders[0].block_timestamp);
            assert_eq!(data[0].block_number, orders[0].block_number);
            assert_eq!(data[0].orderbook_address, orders[0].orderbook_address);
            assert_eq!(data[0].order_bytes, orders[0].order_bytes);
            assert_eq!(data[0].interpreter_address, orders[0].interpreter_address);
            assert_eq!(data[0].store_address, orders[0].store_address);
            assert_eq!(data[0].interpreter_bytecode, orders[0].interpreter_bytecode);
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
            let result = LocalDbQuery::fetch_orders(&callback, FetchOrdersArgs::default()).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_filter_replacement_all() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ = LocalDbQuery::fetch_orders(&callback, FetchOrdersArgs::default()).await;

            let sql = captured_sql.borrow();
            assert!(sql.contains("'all'"), "SQL should contain 'all': {}", *sql);
            assert!(
                !sql.contains("?filter_active"),
                "SQL should not contain placeholder ?filter_active: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_owners"),
                "SQL should not contain placeholder ?filter_owners: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_order_hash"),
                "SQL should not contain placeholder ?filter_order_hash: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_tokens"),
                "SQL should not contain placeholder ?filter_tokens: {}",
                *sql
            );
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_filter_replacement_active() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ = LocalDbQuery::fetch_orders(
                &callback,
                FetchOrdersArgs {
                    filter: FetchOrdersActiveFilter::Active,
                    ..Default::default()
                },
            )
            .await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains("'active'"),
                "SQL should contain 'active': {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_active"),
                "SQL should not contain placeholder ?filter_active: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_owners"),
                "SQL should not contain placeholder ?filter_owners: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_order_hash"),
                "SQL should not contain placeholder ?filter_order_hash: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_tokens"),
                "SQL should not contain placeholder ?filter_tokens: {}",
                *sql
            );
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_filter_replacement_inactive() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ = LocalDbQuery::fetch_orders(
                &callback,
                FetchOrdersArgs {
                    filter: FetchOrdersActiveFilter::Inactive,
                    ..Default::default()
                },
            )
            .await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains("'inactive'"),
                "SQL should contain 'inactive': {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_active"),
                "SQL should not contain placeholder ?filter_active: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_owners"),
                "SQL should not contain placeholder ?filter_owners: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_order_hash"),
                "SQL should not contain placeholder ?filter_order_hash: {}",
                *sql
            );
            assert!(
                !sql.contains("?filter_tokens"),
                "SQL should not contain placeholder ?filter_tokens: {}",
                *sql
            );
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_with_filters_injects_owner_clause() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let args = FetchOrdersArgs {
                filter: FetchOrdersActiveFilter::All,
                owners: vec![
                    Address::from_str("0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap(),
                    Address::from_str("0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB").unwrap(),
                ],
                order_hash: None,
                tokens: vec![],
            };

            let _ = LocalDbQuery::fetch_orders(&callback, args).await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains("AND lower(l.order_owner) IN ('0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb')"),
                "SQL should contain owners filter: {}",
                *sql
            );
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_with_filters_injects_order_hash_clause() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let args = FetchOrdersArgs {
                filter: FetchOrdersActiveFilter::All,
                owners: vec![],
                order_hash: Some(Bytes::from_str("0xabc123").unwrap()),
                tokens: vec![],
            };

            let _ = LocalDbQuery::fetch_orders(&callback, args).await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains(
                    "AND lower(COALESCE(la.order_hash, l.order_hash)) = lower('0xabc123')"
                ),
                "SQL should contain order hash filter: {}",
                *sql
            );
        }

        #[wasm_bindgen_test]
        async fn test_fetch_orders_with_filters_injects_token_clause() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let args = FetchOrdersArgs {
                filter: FetchOrdersActiveFilter::All,
                owners: vec![],
                order_hash: None,
                tokens: vec![
                    Address::from_str("0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap(),
                    Address::from_str("0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB").unwrap(),
                ],
            };

            let _ = LocalDbQuery::fetch_orders(&callback, args).await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains(
                    "AND EXISTS (\n    SELECT 1 FROM order_ios io2\n    WHERE io2.transaction_hash = la.transaction_hash\n      AND io2.log_index = la.log_index\n      AND lower(io2.token) IN ('0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb')\n)"
                ),
                "SQL should contain tokens filter: {}",
                *sql
            );
        }
    }
}
