use super::*;
use alloy::primitives::{Bytes, U256};
use rain_math_float::Float;
use serde::{Deserialize, Serialize};

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDbOrderTrade {
    pub trade_kind: String,
    #[serde(with = "serde_address")]
    pub orderbook_address: Address,
    #[serde(with = "serde_bytes")]
    pub order_hash: Bytes,
    #[serde(with = "serde_address")]
    pub order_owner: Address,
    #[serde(with = "serde_bytes")]
    pub order_nonce: Bytes,
    #[serde(with = "serde_bytes")]
    pub transaction_hash: Bytes,
    pub log_index: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    #[serde(with = "serde_address")]
    pub transaction_sender: Address,
    #[serde(with = "serde_b256")]
    pub input_vault_id: U256,
    #[serde(with = "serde_address")]
    pub input_token: Address,
    pub input_token_name: Option<String>,
    pub input_token_symbol: Option<String>,
    pub input_token_decimals: Option<u8>,
    #[serde(with = "serde_float")]
    pub input_delta: Float,
    #[serde(with = "serde_option_float")]
    pub input_running_balance: Option<Float>,
    #[serde(with = "serde_b256")]
    pub output_vault_id: U256,
    #[serde(with = "serde_address")]
    pub output_token: Address,
    pub output_token_name: Option<String>,
    pub output_token_symbol: Option<String>,
    pub output_token_decimals: Option<u8>,
    #[serde(with = "serde_float")]
    pub output_delta: Float,
    #[serde(with = "serde_option_float")]
    pub output_running_balance: Option<Float>,
    #[serde(with = "serde_bytes")]
    pub trade_id: Bytes,
}

impl LocalDbQuery {
    pub async fn fetch_order_trades(
        db_callback: &js_sys::Function,
        chain_id: u32,
        order_hash: Bytes,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<Vec<LocalDbOrderTrade>, LocalDbQueryError> {
        let order_hash_literal = format!("'{}'", order_hash);

        let filter_start_timestamp = start_timestamp
            .map(|ts| format!("\nAND block_timestamp >= {}\n", ts))
            .unwrap_or_default();
        let filter_end_timestamp = end_timestamp
            .map(|ts| format!("\nAND block_timestamp <= {}\n", ts))
            .unwrap_or_default();

        let sql = QUERY
            .replace("'?order_hash'", &order_hash_literal)
            .replace("'?chain_id'", &chain_id.to_string())
            .replace("?filter_start_timestamp", &filter_start_timestamp)
            .replace("?filter_end_timestamp", &filter_end_timestamp);

        LocalDbQuery::execute_query_json::<Vec<LocalDbOrderTrade>>(db_callback, &sql).await
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
        use alloy::{
            hex::encode_prefixed,
            primitives::{Address, Bytes, B256, U256},
        };
        use rain_math_float::Float;
        use std::str::FromStr;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_fetch_order_trades_parses_rows() {
            let orderbook_address =
                Address::from_str("0x2f209e5b67a33b8fe96e28f24628df6da301c8eb").unwrap();
            let order_owner =
                Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
            let transaction_sender =
                Address::from_str("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
            let order_hash = Bytes::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000abc",
            )
            .unwrap();
            let order_nonce = Bytes::from_str("0x01").unwrap();
            let transaction_hash = Bytes::from_str("0xdeadbeef").unwrap();
            let trade_id = Bytes::from_str("0xfeedface").unwrap();
            let input_delta = Float::from_hex(
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap();
            let input_running_balance = Float::from_hex(
                "0x0000000000000000000000000000000000000000000000000000000000000010",
            )
            .unwrap();
            let output_delta = Float::from_hex(
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap();
            let output_running_balance = Float::from_hex(
                "0x0000000000000000000000000000000000000000000000000000000000000020",
            )
            .unwrap();

            let row = LocalDbOrderTrade {
                trade_kind: "take".into(),
                orderbook_address,
                order_hash: order_hash.clone(),
                order_owner,
                order_nonce,
                transaction_hash,
                log_index: 1,
                block_number: 10,
                block_timestamp: 1000,
                transaction_sender,
                input_vault_id: U256::from(1_u64),
                input_token: Address::from_str("0x00000000000000000000000000000000000000aa")
                    .unwrap(),
                input_token_name: Some("Token In".into()),
                input_token_symbol: Some("TIN".into()),
                input_token_decimals: Some(18),
                input_delta,
                input_running_balance: Some(input_running_balance),
                output_vault_id: U256::from(2_u64),
                output_token: Address::from_str("0x00000000000000000000000000000000000000bb")
                    .unwrap(),
                output_token_name: Some("Token Out".into()),
                output_token_symbol: Some("TOUT".into()),
                output_token_decimals: Some(6),
                output_delta,
                output_running_balance: Some(output_running_balance),
                trade_id,
            };

            let serialized = serde_json::to_string(&vec![row.clone()]).unwrap();
            let expected_input_vault_id = encode_prefixed(B256::from(row.input_vault_id));
            let expected_output_vault_id = encode_prefixed(B256::from(row.output_vault_id));
            assert!(
                serialized.contains(&format!(
                    r#""input_vault_id":"{}""#,
                    expected_input_vault_id
                )),
                "input_vault_id should be hex-prefixed in JSON: {serialized}"
            );
            assert!(
                serialized.contains(&format!(
                    r#""output_vault_id":"{}""#,
                    expected_output_vault_id
                )),
                "output_vault_id should be hex-prefixed in JSON: {serialized}"
            );
            let callback = create_success_callback(&serialized);

            let result =
                LocalDbQuery::fetch_order_trades(&callback, 1, order_hash.clone(), None, None)
                    .await;

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

            let order_hash = Bytes::from_str("0xdeadbeef").unwrap();
            let _ = LocalDbQuery::fetch_order_trades(
                &callback,
                42161,
                order_hash.clone(),
                Some(100),
                Some(200),
            )
            .await;

            let sql = captured_sql.borrow();
            assert!(sql.contains("'0xdeadbeef'"));
            assert!(sql.contains("42161"));
            assert!(sql.contains("block_timestamp >= 100"));
            assert!(sql.contains("block_timestamp <= 200"));
            assert!(!sql.contains("?order_hash"));
        }
    }
}
