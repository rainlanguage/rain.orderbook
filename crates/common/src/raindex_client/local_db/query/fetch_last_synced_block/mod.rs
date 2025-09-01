use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub id: u64,
    pub last_synced_block: u64,
    pub updated_at: Option<String>,
}

impl LocalDbQuery {
    pub async fn fetch_last_synced_block(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<SyncStatusResponse>, LocalDbQueryError> {
        LocalDbQuery::execute_query_with_callback::<Vec<SyncStatusResponse>>(db_callback, QUERY)
            .await
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

        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        async fn test_fetch_last_synced_block() {
            let sync_data = vec![SyncStatusResponse {
                id: 1,
                last_synced_block: 12345,
                updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            }];
            let json_data = serde_json::to_string(&sync_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_last_synced_block(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].id, 1);
            assert_eq!(data[0].last_synced_block, 12345);
            assert_eq!(data[0].updated_at, Some("2024-01-01T00:00:00Z".to_string()));
        }

        #[wasm_bindgen_test]
        async fn test_fetch_last_synced_block_empty() {
            let callback = create_success_callback("[]");

            let result = LocalDbQuery::fetch_last_synced_block(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_last_synced_block_null_timestamp() {
            let sync_data = vec![SyncStatusResponse {
                id: 1,
                last_synced_block: 0,
                updated_at: None,
            }];
            let json_data = serde_json::to_string(&sync_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_last_synced_block(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].id, 1);
            assert_eq!(data[0].last_synced_block, 0);
            assert_eq!(data[0].updated_at, None);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_last_synced_block_max_values() {
            let sync_data = vec![SyncStatusResponse {
                id: u64::MAX,
                last_synced_block: u64::MAX,
                updated_at: Some("2024-12-31T23:59:59Z".to_string()),
            }];
            let json_data = serde_json::to_string(&sync_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_last_synced_block(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].id, u64::MAX);
            assert_eq!(data[0].last_synced_block, u64::MAX);
        }
    }
}
