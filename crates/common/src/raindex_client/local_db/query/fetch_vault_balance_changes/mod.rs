use super::*;
use serde::{Deserialize, Serialize};

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDbVaultBalanceChange {
    #[serde(alias = "chain_id")]
    pub chain_id: u32,
    #[serde(alias = "orderbook_address")]
    pub orderbook_address: String,
    #[serde(alias = "transaction_hash")]
    pub transaction_hash: String,
    #[serde(alias = "log_index")]
    pub log_index: u64,
    #[serde(alias = "block_number")]
    pub block_number: u64,
    #[serde(alias = "block_timestamp")]
    pub block_timestamp: u64,
    pub owner: String,
    #[serde(alias = "change_type")]
    pub change_type: String,
    pub token: String,
    #[serde(alias = "vault_id")]
    pub vault_id: String,
    pub delta: String,
    #[serde(alias = "running_balance")]
    pub running_balance: String,
}

impl LocalDbQuery {
    pub async fn fetch_vault_balance_changes(
        db_callback: &js_sys::Function,
        chain_id: u32,
        orderbook_address: &str,
        vault_id: &str,
        token: &str,
    ) -> Result<Vec<LocalDbVaultBalanceChange>, LocalDbQueryError> {
        let sanitize_literal = |value: &str| value.replace('\'', "''");

        let vault_id = sanitize_literal(&vault_id.trim().to_lowercase());
        let token = sanitize_literal(&token.trim().to_lowercase());
        let orderbook_address = sanitize_literal(&orderbook_address.trim().to_lowercase());

        let sql = QUERY
            .replace("'?vault_id'", &format!("'{}'", vault_id))
            .replace("'?token'", &format!("'{}'", token))
            .replace("'?orderbook_address'", &format!("'{}'", orderbook_address))
            .replace("?chain_id", &chain_id.to_string());

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
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_fetch_vault_balance_changes_parses_data() {
            let change = LocalDbVaultBalanceChange {
                chain_id: 1,
                orderbook_address: "0xob".into(),
                transaction_hash: "0xabc".into(),
                log_index: 2,
                block_number: 42,
                block_timestamp: 1_700_000,
                owner: "0xowner".into(),
                change_type: "DEPOSIT".into(),
                token: "0xtoken".into(),
                vault_id: "0xvault".into(),
                delta: "0x01".into(),
                running_balance: "0x02".into(),
            };

            let callback =
                create_success_callback(&serde_json::to_string(&vec![change.clone()]).unwrap());

            let result = LocalDbQuery::fetch_vault_balance_changes(
                &callback, 1, "0xOB", "0xVAULT", "0xTOKEN",
            )
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

            let _ =
                LocalDbQuery::fetch_vault_balance_changes(&callback, 137, "0xOB", "0xABC", "0xDEF")
                    .await;

            let sql = captured_sql.borrow();
            assert!(sql.contains("'0xabc'"));
            assert!(sql.contains("'0xdef'"));
            assert!(sql.contains("'0xob'"));
            assert!(sql.contains("137"));
            assert!(!sql.contains("?vault_id"));
            assert!(!sql.contains("?token"));
            assert!(!sql.contains("?orderbook_address"));
            assert!(!sql.contains("?chain_id"));
            assert!(sql.contains("ORDER BY vd.block_number DESC"));
        }
    }
}
