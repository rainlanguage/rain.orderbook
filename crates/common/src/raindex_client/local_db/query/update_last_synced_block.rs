use crate::local_db::query::update_last_synced_block::build_update_last_synced_block_stmt;
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};

pub async fn update_last_synced_block<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    block_number: u64,
) -> Result<(), LocalDbQueryError> {
    let stmt = build_update_last_synced_block_stmt(block_number);
    exec.query_text(&stmt).await.map(|_| ())
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let expected_stmt = build_update_last_synced_block_stmt(999);
        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("OK", store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::update_last_synced_block(&exec, 999).await;
        assert!(res.is_ok());
        let captured = store.borrow().clone();
        assert_eq!(captured.0, expected_stmt.sql);
    }

    #[wasm_bindgen_test]
    async fn invalid_response_yields_invalid_response_error() {
        // Return a raw JsValue string instead of WasmEncodedResult
        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let store_clone = store.clone();
        let closure = Closure::wrap(Box::new(
            move |sql: String, params: wasm_bindgen::JsValue| -> wasm_bindgen::JsValue {
                *store_clone.borrow_mut() = (sql, params);
                wasm_bindgen::JsValue::from_str("not-a-wrapper")
            },
        )
            as Box<dyn FnMut(String, wasm_bindgen::JsValue) -> wasm_bindgen::JsValue>);
        let callback: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();

        let exec = JsCallbackExecutor::new(&callback);
        let res = super::update_last_synced_block(&exec, 999).await;
        assert!(matches!(res, Err(LocalDbQueryError::InvalidResponse)));
    }
}
