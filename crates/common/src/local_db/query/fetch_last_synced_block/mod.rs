use serde::{Deserialize, Serialize};

pub const FETCH_LAST_SYNCED_BLOCK_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncStatusResponse {
    pub id: u64,
    #[serde(alias = "lastSyncedBlock")]
    pub last_synced_block: u64,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
}

pub fn fetch_last_synced_block_sql() -> &'static str {
    FETCH_LAST_SYNCED_BLOCK_SQL
}
