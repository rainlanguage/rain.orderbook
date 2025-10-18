use super::*;
use crate::local_db::query::fetch_vault_balance_changes::{
    build_fetch_balance_changes_query, LocalDbVaultBalanceChange,
};

impl LocalDbQuery {
    pub async fn fetch_vault_balance_changes(
        db_callback: &js_sys::Function,
        vault_id: &str,
        token: &str,
    ) -> Result<Vec<LocalDbVaultBalanceChange>, LocalDbQueryError> {
        let sql = build_fetch_balance_changes_query(vault_id, token);
        LocalDbQuery::execute_query_json(db_callback, &sql).await
    }
}
