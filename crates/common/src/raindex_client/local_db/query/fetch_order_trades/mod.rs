use super::*;
use serde::{Deserialize, Serialize};

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDbOrderTrade {
    #[serde(alias = "trade_kind")]
    pub trade_kind: String,
    #[serde(alias = "orderbook_address")]
    pub orderbook_address: String,
    #[serde(alias = "order_hash")]
    pub order_hash: String,
    #[serde(alias = "order_owner")]
    pub order_owner: String,
    #[serde(alias = "order_nonce")]
    pub order_nonce: String,
    #[serde(alias = "transaction_hash")]
    pub transaction_hash: String,
    #[serde(alias = "log_index")]
    pub log_index: u64,
    #[serde(alias = "block_number")]
    pub block_number: u64,
    #[serde(alias = "block_timestamp")]
    pub block_timestamp: u64,
    #[serde(alias = "transaction_sender")]
    pub transaction_sender: String,
    #[serde(alias = "input_vault_id")]
    pub input_vault_id: String,
    #[serde(alias = "input_token")]
    pub input_token: String,
    #[serde(alias = "input_token_name")]
    pub input_token_name: Option<String>,
    #[serde(alias = "input_token_symbol")]
    pub input_token_symbol: Option<String>,
    #[serde(alias = "input_token_decimals")]
    pub input_token_decimals: Option<u8>,
    #[serde(alias = "input_delta")]
    pub input_delta: String,
    #[serde(alias = "input_running_balance")]
    pub input_running_balance: Option<String>,
    #[serde(alias = "output_vault_id")]
    pub output_vault_id: String,
    #[serde(alias = "output_token")]
    pub output_token: String,
    #[serde(alias = "output_token_name")]
    pub output_token_name: Option<String>,
    #[serde(alias = "output_token_symbol")]
    pub output_token_symbol: Option<String>,
    #[serde(alias = "output_token_decimals")]
    pub output_token_decimals: Option<u8>,
    #[serde(alias = "output_delta")]
    pub output_delta: String,
    #[serde(alias = "output_running_balance")]
    pub output_running_balance: Option<String>,
    #[serde(alias = "trade_id")]
    pub trade_id: String,
}

impl LocalDbQuery {
    pub async fn fetch_order_trades(
        db_callback: &js_sys::Function,
        chain_id: u32,
        order_hash: &str,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<Vec<LocalDbOrderTrade>, LocalDbQueryError> {
        let sanitize_literal = |value: &str| value.replace('\'', "''");

        let order_hash = sanitize_literal(&order_hash.trim().to_lowercase());

        let filter_start_timestamp = start_timestamp
            .map(|ts| format!("\nAND block_timestamp >= {}\n", ts))
            .unwrap_or_default();
        let filter_end_timestamp = end_timestamp
            .map(|ts| format!("\nAND block_timestamp <= {}\n", ts))
            .unwrap_or_default();

        let sql = QUERY
            .replace("'?order_hash'", &format!("'{}'", order_hash))
            .replace("'?chain_id'", &chain_id.to_string())
            .replace("?filter_start_timestamp", &filter_start_timestamp)
            .replace("?filter_end_timestamp", &filter_end_timestamp);

        LocalDbQuery::execute_query_json::<Vec<LocalDbOrderTrade>>(db_callback, &sql).await
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
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_fetch_order_trades_parses_rows() {
            let row = LocalDbOrderTrade {
                trade_kind: "take".into(),
                orderbook_address: "0xob".into(),
                order_hash: "0xhash".into(),
                order_owner: "0xowner".into(),
                order_nonce: "1".into(),
                transaction_hash: "0xtx".into(),
                log_index: 1,
                block_number: 10,
                block_timestamp: 1000,
                transaction_sender: "0xsender".into(),
                input_vault_id: "1".into(),
                input_token: "0xinput".into(),
                input_token_name: Some("Token In".into()),
                input_token_symbol: Some("TIN".into()),
                input_token_decimals: Some(18),
                input_delta: "0x1".into(),
                input_running_balance: Some("0x10".into()),
                output_vault_id: "2".into(),
                output_token: "0xoutput".into(),
                output_token_name: Some("Token Out".into()),
                output_token_symbol: Some("TOUT".into()),
                output_token_decimals: Some(6),
                output_delta: "0x2".into(),
                output_running_balance: Some("0x20".into()),
                trade_id: "0xdead".into(),
            };

            let callback =
                create_success_callback(&serde_json::to_string(&vec![row.clone()]).unwrap());

            let result = LocalDbQuery::fetch_order_trades(&callback, 1, "0xABC", None, None).await;

            assert!(result.is_ok());
            let rows = result.unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].trade_id, row.trade_id);
            assert_eq!(rows[0].input_token_symbol, Some("TIN".into()));
        }

        #[wasm_bindgen_test]
        async fn test_fetch_order_trades_replaces_placeholders() {
            let captured_sql = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ =
                LocalDbQuery::fetch_order_trades(&callback, 42161, "0xHASH", Some(100), Some(200))
                    .await;

            let sql = captured_sql.borrow();
            assert!(sql.contains("'0xhash'"));
            assert!(sql.contains("42161"));
            assert!(sql.contains("block_timestamp >= 100"));
            assert!(sql.contains("block_timestamp <= 200"));
            assert!(!sql.contains("?order_hash"));
        }
    }
}
