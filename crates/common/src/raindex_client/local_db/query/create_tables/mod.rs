use super::*;
use crate::local_db::query::create_tables::create_tables_sql;

impl LocalDbQuery {
    pub async fn create_tables(db_callback: &js_sys::Function) -> Result<(), LocalDbQueryError> {
        LocalDbQuery::execute_query_text(db_callback, create_tables_sql())
            .await
            .map(|_| ())
    }
}

pub use crate::local_db::query::create_tables::REQUIRED_TABLES;

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_raw_sql_exactly() {
        let expected_sql = create_tables_sql();
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("OK", store.clone());
        let res = LocalDbQuery::create_tables(&callback).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone(), expected_sql);
    }
}
