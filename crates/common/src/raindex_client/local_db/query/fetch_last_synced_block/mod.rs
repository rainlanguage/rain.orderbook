use super::*;

pub const FETCH_LAST_SYNCED_BLOCK_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SyncStatusResponse {
    pub id: u64,
    #[serde(alias = "lastSyncedBlock")]
    pub last_synced_block: u64,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(SyncStatusResponse);

impl LocalDbQuery {
    pub async fn fetch_last_synced_block(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<SyncStatusResponse>, LocalDbQueryError> {
        LocalDbQuery::execute_query_json::<Vec<SyncStatusResponse>>(
            db_callback,
            FETCH_LAST_SYNCED_BLOCK_SQL,
        )
        .await
    }
}

#[wasm_export]
impl LocalDb {
    #[wasm_export(
        js_name = "getSyncStatus",
        unchecked_return_type = "SyncStatusResponse[]"
    )]
    pub async fn get_sync_status(
        &self,
        db_callback: js_sys::Function,
    ) -> Result<Vec<SyncStatusResponse>, RaindexError> {
        LocalDbQuery::fetch_last_synced_block(&db_callback)
            .await
            .map_err(|e| RaindexError::LocalDbError(LocalDbError::LocalDbQueryError(e)))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::create_success_callback;
        use wasm_bindgen_test::*;

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
