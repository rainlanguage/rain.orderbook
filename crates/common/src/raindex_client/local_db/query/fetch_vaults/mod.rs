use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbVault {
    pub vault_id: String,
    pub token: String,
    pub owner: String,
    pub balance: String,
    pub input_order_hashes: Option<String>,
    pub output_order_hashes: Option<String>,
}

impl LocalDbQuery {
    pub async fn fetch_vaults(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<LocalDbVault>, LocalDbQueryError> {
        LocalDbQuery::execute_query_json::<Vec<LocalDbVault>>(db_callback, QUERY).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::create_success_callback;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_fetch_vaults_parses_data() {
            let vaults = vec![
                LocalDbVault {
                    vault_id: "0x01".into(),
                    token: "0xaaa".into(),
                    owner: "0x1111111111111111111111111111111111111111".into(),
                    balance: "0x10".into(),
                    input_order_hashes: Some(
                        "0xabc0000000000000000000000000000000000000000000000000000000000001".into(),
                    ),
                    output_order_hashes: Some(
                        "0xdef0000000000000000000000000000000000000000000000000000000000002".into(),
                    ),
                },
                LocalDbVault {
                    vault_id: "0x02".into(),
                    token: "0xbbb".into(),
                    owner: "0x2222222222222222222222222222222222222222".into(),
                    balance: "0x0".into(),
                    input_order_hashes: None,
                    output_order_hashes: None,
                },
            ];
            let json_data = serde_json::to_string(&vaults).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_vaults(&callback).await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].vault_id, vaults[0].vault_id);
            assert_eq!(data[0].token, vaults[0].token);
            assert_eq!(data[0].owner, vaults[0].owner);
            assert_eq!(data[0].balance, vaults[0].balance);
            assert_eq!(data[0].input_order_hashes, vaults[0].input_order_hashes);
            assert_eq!(data[0].output_order_hashes, vaults[0].output_order_hashes);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_vaults_empty() {
            let callback = create_success_callback("[]");
            let result = LocalDbQuery::fetch_vaults(&callback).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);
        }
    }
}
