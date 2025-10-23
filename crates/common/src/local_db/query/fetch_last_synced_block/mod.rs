use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

pub const FETCH_LAST_SYNCED_BLOCK_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

use crate::local_db::query::SqlStatement;

pub fn fetch_last_synced_block_stmt() -> SqlStatement {
    SqlStatement::new(FETCH_LAST_SYNCED_BLOCK_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stmt_is_static_and_param_free() {
        let stmt = fetch_last_synced_block_stmt();
        assert_eq!(stmt.sql, FETCH_LAST_SYNCED_BLOCK_SQL);
        assert!(stmt.params.is_empty());
        assert!(stmt.sql.to_lowercase().contains("from sync_status"));
    }
}
