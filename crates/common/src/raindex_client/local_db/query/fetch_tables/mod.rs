use super::*;
use crate::local_db::query::fetch_tables::{fetch_tables_sql, TableResponse};

impl LocalDbQuery {
    pub async fn fetch_all_tables(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<TableResponse>, LocalDbQueryError> {
        LocalDbQuery::execute_query_json(db_callback, fetch_tables_sql()).await
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_raw_sql_exactly() {
        let expected_sql = fetch_tables_sql();
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());

        let res = LocalDbQuery::fetch_all_tables(&callback).await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }

    #[wasm_bindgen_test]
    async fn invalid_response_yields_invalid_response_error() {
        // Return a raw JsValue string instead of WasmEncodedResult
        let store = Rc::new(RefCell::new(String::new()));
        let store_clone = store.clone();
        let closure = Closure::wrap(Box::new(move |sql: String| -> wasm_bindgen::JsValue {
            *store_clone.borrow_mut() = sql;
            wasm_bindgen::JsValue::from_str("not-a-wrapper")
        })
            as Box<dyn FnMut(String) -> wasm_bindgen::JsValue>);
        let callback: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();

        let res = LocalDbQuery::fetch_all_tables(&callback).await;
        assert!(matches!(res, Err(LocalDbQueryError::InvalidResponse)));
    }
}
