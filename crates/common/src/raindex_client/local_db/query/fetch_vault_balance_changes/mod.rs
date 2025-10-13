use super::*;
use alloy::primitives::{Bytes, U256};
use rain_math_float::Float;
use serde::{Deserialize, Serialize};

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDbVaultBalanceChange {
    #[serde(with = "serde_bytes")]
    pub transaction_hash: Bytes,
    pub log_index: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    #[serde(with = "serde_address")]
    pub owner: Address,
    pub change_type: String,
    #[serde(with = "serde_address")]
    pub token: Address,
    #[serde(with = "serde_u256")]
    pub vault_id: U256,
    #[serde(with = "serde_float")]
    pub delta: Float,
    #[serde(with = "serde_float")]
    pub running_balance: Float,
}

impl LocalDbQuery {
    pub async fn fetch_vault_balance_changes(
        db_callback: &js_sys::Function,
        vault_id: &str,
        token: &str,
    ) -> Result<Vec<LocalDbVaultBalanceChange>, LocalDbQueryError> {
        let sanitize_literal = |value: &str| value.replace('\'', "''");

        let vault_id = sanitize_literal(&vault_id.trim().to_lowercase());
        let token = sanitize_literal(&token.trim().to_lowercase());

        let sql = QUERY
            .replace("'?vault_id'", &format!("'{}'", vault_id))
            .replace("'?token'", &format!("'{}'", token));

        LocalDbQuery::execute_query_json::<Vec<LocalDbVaultBalanceChange>>(db_callback, &sql).await
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
        use alloy::primitives::{Address, Bytes, U256};
        use rain_math_float::Float;
        use std::str::FromStr;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_fetch_vault_balance_changes_parses_data() {
            let transaction_hash = Bytes::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000abc",
            )
            .unwrap();
            let owner = Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
            let token = Address::from_str("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
            let vault_id = U256::from(0x10_u64);
            let delta = Float::from_hex(
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap();
            let running_balance = Float::from_hex(
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap();

            let change = LocalDbVaultBalanceChange {
                transaction_hash: transaction_hash.clone(),
                log_index: 2,
                block_number: 42,
                block_timestamp: 1_700_000,
                owner,
                change_type: "DEPOSIT".into(),
                token,
                vault_id,
                delta,
                running_balance,
            };

            let callback =
                create_success_callback(&serde_json::to_string(&vec![change.clone()]).unwrap());

            let result = LocalDbQuery::fetch_vault_balance_changes(&callback, "0xVAULT", "0xTOKEN")
                .await
                .expect("query should succeed");

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].transaction_hash, change.transaction_hash);
            assert_eq!(result[0].change_type, change.change_type);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_vault_balance_changes_replaces_placeholders() {
            let captured_sql = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let _ = LocalDbQuery::fetch_vault_balance_changes(&callback, "0xABC", "0xDEF").await;

            let sql = captured_sql.borrow();
            assert!(sql.contains("'0xabc'"));
            assert!(sql.contains("'0xdef'"));
            assert!(!sql.contains("?vault_id"));
            assert!(!sql.contains("?token"));
            assert!(sql.contains("ORDER BY vd.block_number DESC"));
        }
    }
}
