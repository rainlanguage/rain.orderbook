pub mod clear_tables;
pub mod create_tables;
pub mod fetch_erc20_tokens_by_addresses;
pub mod fetch_last_synced_block;
pub mod fetch_order_trades;
pub mod fetch_order_trades_count;
pub mod fetch_orders;
pub mod fetch_store_addresses;
pub mod fetch_tables;
pub mod fetch_vault;
pub mod fetch_vault_balance_changes;
pub mod fetch_vaults;
pub mod update_last_synced_block;

use crate::local_db::{
    query::{FromDbJson, LocalDbQueryError},
    LocalDb, LocalDbError,
};
use wasm_bindgen_utils::prelude::{
    js_sys, serde_wasm_bindgen, wasm_bindgen_futures::JsFuture, JsValue, *,
};
use wasm_bindgen_utils::result::WasmEncodedResult;

pub struct LocalDbQuery;

impl LocalDbQuery {
    async fn execute_query_raw(
        callback: &js_sys::Function,
        sql: &str,
    ) -> Result<String, LocalDbQueryError> {
        let result = callback
            .call1(&JsValue::NULL, &JsValue::from_str(sql))
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

    pub async fn execute_query_json<T>(
        callback: &js_sys::Function,
        sql: &str,
    ) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson + Send,
    {
        let value = Self::execute_query_raw(callback, sql).await?;
        serde_json::from_str(&value)
            .map_err(|err| LocalDbQueryError::deserialization(err.to_string()))
    }

    pub async fn execute_query_text(
        callback: &js_sys::Function,
        sql: &str,
    ) -> Result<String, LocalDbQueryError> {
        Self::execute_query_raw(callback, sql).await
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
        use wasm_bindgen_test::*;
        use wasm_bindgen_utils::prelude::serde_wasm_bindgen;
        use wasm_bindgen_utils::prelude::JsValue;
        use wasm_bindgen_utils::result::{WasmEncodedError, WasmEncodedResult};

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
            store: std::rc::Rc<std::cell::RefCell<String>>,
        ) -> Function {
            use wasm_bindgen::prelude::Closure;
            use wasm_bindgen::JsCast;

            let response = response.to_string();
            let store_clone = store.clone();
            let closure = Closure::wrap(Box::new(move |sql: String| -> wasm_bindgen::JsValue {
                *store_clone.borrow_mut() = sql;
                let result = WasmEncodedResult::Success::<String> {
                    value: response.clone(),
                    error: None,
                };
                serde_wasm_bindgen::to_value(&result).unwrap()
            })
                as Box<dyn FnMut(String) -> wasm_bindgen::JsValue>);

            let func: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();
            func
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_success_case() {
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

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT * FROM users").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].name, "Alice");
            assert_eq!(data[1].name, "Bob");
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_raw_success() {
            let callback = create_success_callback("\"ok\"");
            let val = LocalDbQuery::execute_query_raw(&callback, "SELECT 1")
                .await
                .unwrap();
            assert_eq!(val, "\"ok\"");
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_success() {
            let callback = create_success_callback("text-result");
            let val = LocalDbQuery::execute_query_text(&callback, "SELECT 1")
                .await
                .unwrap();
            assert_eq!(val, "text-result");
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_raw_callback_throws() {
            // callback that throws synchronously
            let callback = Function::new_with_args("sql", "throw new Error('boom')");
            let err = LocalDbQuery::execute_query_raw(&callback, "SELECT 1")
                .await
                .err()
                .unwrap();
            match err {
                LocalDbQueryError::Database { .. } => {}
                other => panic!("unexpected error variant: {:?}", other),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_raw_promise_rejects() {
            // callback returns a rejected Promise
            let callback = Function::new_with_args("sql", "return Promise.reject('rejected')");
            let err = LocalDbQuery::execute_query_raw(&callback, "SELECT 1")
                .await
                .err()
                .unwrap();
            match err {
                LocalDbQueryError::Database { .. } => {}
                other => panic!("unexpected error variant: {:?}", other),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_json_invalid_response() {
            // returns a plain string instead of WasmEncodedResult
            let callback = Function::new_with_args("sql", "return 'not-a-wrapper'");
            let res: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT 1").await;
            assert!(matches!(res, Err(LocalDbQueryError::InvalidResponse)));
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_json_deserialization_error() {
            // Success wrapper but invalid JSON payload
            let store = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
            let store_clone = store.clone();
            let closure =
                wasm_bindgen::prelude::Closure::wrap(Box::new(move |sql: String| -> JsValue {
                    *store_clone.borrow_mut() = sql;
                    let result: WasmEncodedResult<String> = WasmEncodedResult::Success {
                        value: "not-json".to_string(),
                        error: None,
                    };
                    serde_wasm_bindgen::to_value(&result).unwrap()
                })
                    as Box<dyn FnMut(String) -> JsValue>);
            let callback: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let res: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT 1").await;
            assert!(matches!(
                res,
                Err(LocalDbQueryError::Deserialization { .. })
            ));
        }
    }
}
