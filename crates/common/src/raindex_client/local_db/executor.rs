use super::*;
use crate::local_db::query::{FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement};
use async_trait::async_trait;

pub struct JsCallbackExecutor<'a> {
    callback: &'a js_sys::Function,
}

impl<'a> JsCallbackExecutor<'a> {
    pub fn new(callback: &'a js_sys::Function) -> Self {
        Self { callback }
    }
}

#[async_trait(?Send)]
impl<'a> LocalDbQueryExecutor for JsCallbackExecutor<'a> {
    async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
        use wasm_bindgen_utils::prelude::{js_sys, wasm_bindgen_futures::JsFuture, JsValue};
        use wasm_bindgen_utils::{prelude::serde_wasm_bindgen, result::WasmEncodedResult};

        // If there are no parameters, pass `undefined` to the JS callback
        // instead of an empty array to match the SDK's expected semantics.
        let js_params_val = if stmt.params.is_empty() {
            JsValue::UNDEFINED
        } else {
            serde_wasm_bindgen::to_value(&stmt.as_js_params())
                .map_err(|e| LocalDbQueryError::database(format!("encode params: {:?}", e)))?
        };

        let result = self
            .callback
            .call2(
                &JsValue::NULL,
                &JsValue::from_str(&stmt.sql),
                &js_params_val,
            )
            .map_err(|e| {
                LocalDbQueryError::database(format!(
                    "JavaScript callback invocation failed: {:?}",
                    e
                ))
            })?;

        let promise = js_sys::Promise::resolve(&result);
        let future = JsFuture::from(promise);
        let js_result = future.await.map_err(|e| {
            LocalDbQueryError::database(format!("Promise resolution failed: {:?}", e))
        })?;

        let wasm_result: WasmEncodedResult<String> = serde_wasm_bindgen::from_value(js_result)
            .map_err(|_| LocalDbQueryError::invalid_response())?;

        match wasm_result {
            WasmEncodedResult::Success { value, .. } => Ok(value),
            WasmEncodedResult::Err { error, .. } => {
                Err(LocalDbQueryError::database(error.readable_msg))
            }
        }
    }

    async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson,
    {
        let value = self.query_text(stmt).await?;
        serde_json::from_str(&value)
            .map_err(|err| LocalDbQueryError::deserialization(err.to_string()))
    }
}

#[cfg(test)]
pub mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    pub use wasm_tests::create_sql_capturing_callback;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use js_sys::Function;
        use serde::{Deserialize, Serialize};
        use wasm_bindgen::JsCast;
        use wasm_bindgen_test::*;
        use wasm_bindgen_utils::prelude::serde_wasm_bindgen;
        use wasm_bindgen_utils::prelude::JsValue;
        use wasm_bindgen_utils::result::WasmEncodedResult;

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TestData {
            id: u32,
            name: String,
        }

        pub fn create_success_callback(response: &str) -> Function {
            let success = WasmEncodedResult::Success::<String> {
                value: response.to_string(),
                error: None,
            };
            let js_value = serde_wasm_bindgen::to_value(&success).unwrap();
            Function::new_no_args(&format!(
                "return {}",
                js_sys::JSON::stringify(&js_value)
                    .unwrap()
                    .as_string()
                    .unwrap()
            ))
        }

        pub fn create_sql_capturing_callback(
            response: &str,
            store: std::rc::Rc<std::cell::RefCell<(String, JsValue)>>,
        ) -> Function {
            use wasm_bindgen::prelude::Closure;

            let response = response.to_string();
            let store_clone = store.clone();
            let closure = Closure::wrap(Box::new(
                move |sql: String, params: JsValue| -> wasm_bindgen::JsValue {
                    *store_clone.borrow_mut() = (sql, params);
                    let result = WasmEncodedResult::Success::<String> {
                        value: response.clone(),
                        error: None,
                    };
                    serde_wasm_bindgen::to_value(&result).unwrap()
                },
            )
                as Box<dyn FnMut(String, JsValue) -> wasm_bindgen::JsValue>);

            let func: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();
            func
        }

        #[wasm_bindgen_test]
        async fn test_query_json_success_case() {
            let test_data = vec![
                TestData {
                    id: 1,
                    name: "Alice".to_string(),
                },
                TestData {
                    id: 2,
                    name: "Bob".to_string(),
                },
            ];
            let json_data = serde_json::to_string(&test_data).unwrap();
            let callback = create_success_callback(&json_data);
            let exec = JsCallbackExecutor::new(&callback);

            let result: Result<Vec<TestData>, LocalDbQueryError> = exec
                .query_json(&SqlStatement::new("SELECT * FROM users"))
                .await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].name, "Alice");
            assert_eq!(data[1].name, "Bob");
        }

        #[wasm_bindgen_test]
        async fn test_query_text_success() {
            let callback = create_success_callback("text-result");
            let exec = JsCallbackExecutor::new(&callback);
            let val = exec
                .query_text(&SqlStatement::new("SELECT 1"))
                .await
                .unwrap();
            assert_eq!(val, "text-result");
        }

        #[wasm_bindgen_test]
        async fn passes_undefined_params_when_empty() {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::JsValue;

            let store = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let callback = create_sql_capturing_callback("OK", store.clone());
            let exec = JsCallbackExecutor::new(&callback);

            let _ = exec
                .query_text(&SqlStatement::new("SELECT 42"))
                .await
                .unwrap();

            let (_, captured_params) = store.borrow().clone();
            assert!(captured_params.is_undefined());
        }

        #[wasm_bindgen_test]
        async fn test_callback_throws() {
            // callback that throws synchronously
            let callback = Function::new_with_args("sql, params", "throw new Error('boom')");
            let exec = JsCallbackExecutor::new(&callback);
            let err = exec
                .query_text(&SqlStatement::new("SELECT 1"))
                .await
                .err()
                .unwrap();
            match err {
                LocalDbQueryError::Database { .. } => {}
                other => panic!("unexpected error variant: {:?}", other),
            }
        }

        #[wasm_bindgen_test]
        async fn test_promise_rejects() {
            // callback returns a rejected Promise
            let callback =
                Function::new_with_args("sql, params", "return Promise.reject('rejected')");
            let exec = JsCallbackExecutor::new(&callback);
            let err = exec
                .query_text(&SqlStatement::new("SELECT 1"))
                .await
                .err()
                .unwrap();
            match err {
                LocalDbQueryError::Database { .. } => {}
                other => panic!("unexpected error variant: {:?}", other),
            }
        }

        #[wasm_bindgen_test]
        async fn test_invalid_wrapper_yields_invalid_response() {
            // returns a plain string instead of WasmEncodedResult
            let callback = Function::new_with_args("sql, params", "return 'not-a-wrapper'");
            let exec = JsCallbackExecutor::new(&callback);
            let res: Result<Vec<TestData>, LocalDbQueryError> =
                exec.query_json(&SqlStatement::new("SELECT 1")).await;
            assert!(matches!(res, Err(LocalDbQueryError::InvalidResponse)));
        }

        #[wasm_bindgen_test]
        async fn test_deserialization_error() {
            // Success wrapper but invalid JSON payload
            use wasm_bindgen_utils::result::WasmEncodedResult;
            let store =
                std::rc::Rc::new(std::cell::RefCell::new((String::new(), JsValue::UNDEFINED)));
            let store_clone = store.clone();
            let closure = wasm_bindgen::prelude::Closure::wrap(Box::new(
                move |sql: String, params: JsValue| -> JsValue {
                    *store_clone.borrow_mut() = (sql, params);
                    let result: WasmEncodedResult<String> = WasmEncodedResult::Success {
                        value: "not-json".to_string(),
                        error: None,
                    };
                    serde_wasm_bindgen::to_value(&result).unwrap()
                },
            )
                as Box<dyn FnMut(String, JsValue) -> JsValue>);
            let callback: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let exec = JsCallbackExecutor::new(&callback);
            let res: Result<Vec<TestData>, LocalDbQueryError> =
                exec.query_json(&SqlStatement::new("SELECT 1")).await;
            assert!(matches!(
                res,
                Err(LocalDbQueryError::Deserialization { .. })
            ));
        }
    }
}
