use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbVault {
    #[serde(alias = "vault_id")]
    pub vault_id: String,
    pub token: String,
    pub owner: String,
    #[serde(alias = "orderbookAddress")]
    pub orderbook_address: String,
    pub balance: String,
    #[serde(alias = "input_order_hashes")]
    pub input_order_hashes: Option<String>,
    #[serde(alias = "output_order_hashes")]
    pub output_order_hashes: Option<String>,
}

impl LocalDbQuery {
    pub async fn fetch_vault(
        db_callback: &js_sys::Function,
        vault_id: &str,
        token: &str,
    ) -> Result<Option<LocalDbVault>, LocalDbQueryError> {
        let sql = QUERY
            .replace("'?vault_id'", &format!("'{}'", vault_id))
            .replace("'?token'", &format!("'{}'", token));

        let rows: Vec<LocalDbVault> = LocalDbQuery::execute_query_json(db_callback, &sql).await?;
        Ok(rows.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::create_success_callback;
        use std::rc::Rc;
        use std::cell::RefCell;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_fetch_vault_parses_data() {
            let vault = LocalDbVault {
                vault_id: "0x01".into(),
                token: "0xaaa".into(),
                owner: "0x1111111111111111111111111111111111111111".into(),
                orderbook_address: "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB".into(),
                balance: "0x10".into(),
                input_order_hashes: Some(
                    "0xabc0000000000000000000000000000000000000000000000000000000000001".into(),
                ),
                output_order_hashes: None,
            };
            let json_data = serde_json::to_string(&vec![vault.clone()]).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_vault(&callback, "0x01", "0xaaa").await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert!(data.is_some());
            let data = data.unwrap();
            assert_eq!(data.vault_id, vault.vault_id);
            assert_eq!(data.token, vault.token);
            assert_eq!(data.owner, vault.owner);
            assert_eq!(data.orderbook_address, vault.orderbook_address);
            assert_eq!(data.balance, vault.balance);
            assert_eq!(data.input_order_hashes, vault.input_order_hashes);
            assert_eq!(data.output_order_hashes, vault.output_order_hashes);
        }

        fn create_sql_capturing_callback(response_json: &str, captured: Rc<RefCell<String>>) -> js_sys::Function {
            let success_result = WasmEncodedResult::Success::<String> {
                value: response_json.to_string(),
                error: None,
            };
            let js_value = serde_wasm_bindgen::to_value(&success_result).unwrap();

            js_sys::Function::new_with_args(
                "sql",
                &format!(
                    "captured = sql; return {};",
                    js_sys::JSON::stringify(&js_value)
                        .unwrap()
                        .as_string()
                        .unwrap()
                ),
            )
        }

        #[wasm_bindgen_test]
        async fn test_fetch_vault_replaces_placeholders() {
            let captured = Rc::new(RefCell::new(String::new()));
            // Provide empty result array
            let callback = create_sql_capturing_callback("[]", captured.clone());

            let _ = LocalDbQuery::fetch_vault(&callback, "0xdead", "0xbeef").await;

            let sql = captured.borrow();
            assert!(sql.contains("'0xdead'"));
            assert!(sql.contains("'0xbeef'"));
            assert!(!sql.contains("?vault_id"));
            assert!(!sql.contains("?token"));
        }
    }
}

