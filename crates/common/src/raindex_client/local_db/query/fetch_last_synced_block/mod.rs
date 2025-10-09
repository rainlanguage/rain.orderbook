use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SyncStatusResponse {
    #[serde(alias = "chainId")]
    pub chain_id: u64,
    #[serde(alias = "orderbookAddress")]
    pub orderbook_address: String,
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
        chain_id: u64,
        orderbook_address: &str,
    ) -> Result<Vec<SyncStatusResponse>, LocalDbQueryError> {
        let escaped_address = orderbook_address.replace('\'', "''");
        let sql = QUERY
            .replace("?chain_id", &chain_id.to_string())
            .replace("?orderbook_address", &escaped_address);

        LocalDbQuery::execute_query_json::<Vec<SyncStatusResponse>>(db_callback, &sql).await
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
        chain_id: u32,
        orderbook_address: String,
    ) -> Result<Vec<SyncStatusResponse>, RaindexError> {
        LocalDbQuery::fetch_last_synced_block(&db_callback, chain_id as u64, &orderbook_address)
            .await
            .map_err(|e| RaindexError::LocalDbError(LocalDbError::LocalDbQueryError(e)))
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
        async fn test_fetch_last_synced_block() {
            let sync_data = vec![SyncStatusResponse {
                chain_id: 1,
                orderbook_address: "0xabc".to_string(),
                last_synced_block: 12345,
                updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            }];
            let json_data = serde_json::to_string(&sync_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_last_synced_block(&callback, 1, "0xabc").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].chain_id, 1);
            assert_eq!(data[0].orderbook_address, "0xabc");
            assert_eq!(data[0].last_synced_block, 12345);
            assert_eq!(data[0].updated_at, Some("2024-01-01T00:00:00Z".to_string()));
        }

        #[wasm_bindgen_test]
        async fn test_fetch_last_synced_block_empty() {
            let callback = create_success_callback("[]");

            let result = LocalDbQuery::fetch_last_synced_block(&callback, 1, "0xabc").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_last_synced_block_null_timestamp() {
            let sync_data = vec![SyncStatusResponse {
                chain_id: 1,
                orderbook_address: "0xabc".to_string(),
                last_synced_block: 0,
                updated_at: None,
            }];
            let json_data = serde_json::to_string(&sync_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_last_synced_block(&callback, 1, "0xabc").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].chain_id, 1);
            assert_eq!(data[0].last_synced_block, 0);
            assert_eq!(data[0].updated_at, None);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_last_synced_block_max_values() {
            let sync_data = vec![SyncStatusResponse {
                chain_id: u64::MAX,
                orderbook_address: "0xabc".to_string(),
                last_synced_block: u64::MAX,
                updated_at: Some("2024-12-31T23:59:59Z".to_string()),
            }];
            let json_data = serde_json::to_string(&sync_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_last_synced_block(&callback, u64::MAX, "0xabc").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].chain_id, u64::MAX);
            assert_eq!(data[0].last_synced_block, u64::MAX);
        }
    }
}
