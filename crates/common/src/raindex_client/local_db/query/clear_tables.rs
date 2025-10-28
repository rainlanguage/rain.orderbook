use crate::local_db::query::clear_tables::clear_tables_sql;
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::{LocalDb, LocalDbError};
use wasm_bindgen_utils::prelude::*;

pub async fn clear_tables<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
) -> Result<(), LocalDbQueryError> {
    exec.query_text(clear_tables_sql()).await.map(|_| ())
}

#[wasm_export]
impl LocalDb {
    /// Clears all local database tables using the provided JS query callback.
    #[wasm_export(js_name = "clearTables", unchecked_return_type = "void")]
    pub async fn clear_tables_wasm(
        &self,
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
    ) -> Result<(), LocalDbError> {
        let exec = crate::raindex_client::local_db::executor::JsCallbackExecutor::new(&db_callback);
        clear_tables(&exec).await.map_err(LocalDbError::from)
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_raw_sql_exactly() {
        let expected_sql = clear_tables_sql();
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("OK", store.clone());
        let exec = JsCallbackExecutor::new(&callback);
        let res = super::clear_tables(&exec).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone(), expected_sql);
    }
}
