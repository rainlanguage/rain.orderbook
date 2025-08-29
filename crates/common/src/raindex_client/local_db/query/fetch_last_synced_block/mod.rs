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
