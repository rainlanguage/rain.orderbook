use super::*;
use crate::local_db::query::fetch_vault_balance_changes::{
    build_fetch_balance_changes_query, LocalDbVaultBalanceChange,
};
use crate::local_db::query::LocalDbQueryExecutor;

impl LocalDbQuery {
    pub async fn fetch_vault_balance_changes<E: LocalDbQueryExecutor + ?Sized>(
        exec: &E,
        vault_id: &str,
        token: &str,
    ) -> Result<Vec<LocalDbVaultBalanceChange>, LocalDbQueryError> {
        let sql = build_fetch_balance_changes_query(vault_id, token);
        exec.query_json(&sql).await
    }
}
