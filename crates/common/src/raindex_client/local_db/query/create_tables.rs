use crate::local_db::query::create_tables::create_tables_sql;
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};

pub async fn create_tables<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
) -> Result<(), LocalDbQueryError> {
    exec.query_text(create_tables_sql()).await.map(|_| ())
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
        let expected_sql = create_tables_sql();
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("OK", store.clone());
        let exec = JsCallbackExecutor::new(&callback);
        let res = super::create_tables(&exec).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone(), expected_sql);
    }
}
