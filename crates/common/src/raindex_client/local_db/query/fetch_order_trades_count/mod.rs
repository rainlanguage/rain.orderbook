use super::*;
use alloy::primitives::Bytes;
use serde::{Deserialize, Serialize};

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalDbTradeCountRow {
    trade_count: u64,
}

impl LocalDbQuery {
    pub async fn fetch_order_trades_count(
        db_callback: &js_sys::Function,
        order_hash: Bytes,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<u64, LocalDbQueryError> {
        let order_hash_literal = format!("'{}'", order_hash);

        let filter_start_timestamp = start_timestamp
            .map(|ts| format!("\nAND block_timestamp >= {}\n", ts))
            .unwrap_or_default();
        let filter_end_timestamp = end_timestamp
            .map(|ts| format!("\nAND block_timestamp <= {}\n", ts))
            .unwrap_or_default();

        let sql = QUERY
            .replace("'?order_hash'", &order_hash_literal)
            .replace("?filter_start_timestamp", &filter_start_timestamp)
            .replace("?filter_end_timestamp", &filter_end_timestamp);

        let rows = LocalDbQuery::execute_query_json::<Vec<LocalDbTradeCountRow>>(db_callback, &sql)
            .await?;

        Ok(rows.first().map(|row| row.trade_count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::{
            create_sql_capturing_callback, create_success_callback,
        };
        use std::str::FromStr;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_fetch_order_trades_count_parses_value() {
            let callback = create_success_callback("[{\"trade_count\": 7}]");

            let order_hash = Bytes::from_str("0x0abc").unwrap();
            let result =
                LocalDbQuery::fetch_order_trades_count(&callback, order_hash, None, None).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 7);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_order_trades_count_defaults_to_zero() {
            let callback = create_success_callback("[]");

            let order_hash = Bytes::from_str("0x0abc").unwrap();
            let result =
                LocalDbQuery::fetch_order_trades_count(&callback, order_hash, None, None).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_order_trades_count_replaces_placeholder() {
            let captured_sql = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
            let callback =
                create_sql_capturing_callback("[{\"trade_count\":0}]", captured_sql.clone());

            let order_hash = Bytes::from_str("0xabcdef").unwrap();
            let _ = LocalDbQuery::fetch_order_trades_count(&callback, order_hash, Some(1), Some(2))
                .await;

            let sql = captured_sql.borrow();
            assert!(sql.contains("'0xabcdef'"));
            assert!(!sql.contains("?order_hash"));
            assert!(sql.contains("block_timestamp >= 1"));
            assert!(sql.contains("block_timestamp <= 2"));
        }
    }
}
