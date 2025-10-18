use super::*;
use crate::local_db::query::fetch_last_synced_block::{
    fetch_last_synced_block_sql, SyncStatusResponse,
};

impl LocalDbQuery {
    pub async fn fetch_last_synced_block(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<SyncStatusResponse>, LocalDbQueryError> {
        LocalDbQuery::execute_query_json(db_callback, fetch_last_synced_block_sql()).await
    }
}

#[wasm_export]
impl LocalDb {
    /// Returns the sync status rows from the local DB (via provided JS query callback)
    #[wasm_export(js_name = "getSyncStatus")]
    pub async fn get_sync_status(
        &self,
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
    ) -> Result<Vec<SyncStatusResponse>, LocalDbError> {
        LocalDbQuery::fetch_last_synced_block(&db_callback)
            .await
            .map_err(LocalDbError::from)
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_raw_sql_exactly() {
        let expected_sql = fetch_last_synced_block_sql();
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let res = LocalDbQuery::fetch_last_synced_block(&callback).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone(), expected_sql);
    }
}
