pub mod create_tables;
pub mod fetch_last_synced_block;
pub mod fetch_tables;
pub mod update_last_synced_block;

use super::*;
use wasm_bindgen_utils::prelude::wasm_bindgen_futures::JsFuture;

pub struct LocalDbQuery;

#[derive(Error, Debug)]
pub enum LocalDbQueryError {
    #[error("JavaScript callback invocation failed: {0}")]
    CallbackError(String),

    #[error("Promise resolution failed: {0}")]
    PromiseError(String),

    #[error("JSON deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Database operation failed: {message}")]
    DatabaseError { message: String },

    #[error("Invalid response format from database")]
    InvalidResponse,
}

impl LocalDbQuery {
    pub async fn execute_query_with_callback<T>(
        callback: &js_sys::Function,
        sql: &str,
    ) -> Result<T, LocalDbQueryError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let result = callback
            .call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(sql),
            )
            .map_err(|e| LocalDbQueryError::CallbackError(format!("{:?}", e)))?;

        let promise = js_sys::Promise::resolve(&result);
        let future = JsFuture::from(promise);

        let js_result = future
            .await
            .map_err(|e| LocalDbQueryError::PromiseError(format!("{:?}", e)))?;

        let wasm_result: WasmEncodedResult<String> = serde_wasm_bindgen::from_value(js_result)
            .map_err(|_| LocalDbQueryError::InvalidResponse)?;

        match wasm_result {
            WasmEncodedResult::Success { value, .. } => {
                serde_json::from_str(&value).map_err(LocalDbQueryError::JsonError)
            }
            WasmEncodedResult::Err { error, .. } => Err(LocalDbQueryError::DatabaseError {
                message: error.readable_msg,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use js_sys::Function;
        use serde::{Deserialize, Serialize};
        use wasm_bindgen_test::*;

        wasm_bindgen_test_configure!(run_in_browser);

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TestData {
            id: u32,
            name: String,
        }

        fn create_success_callback(json_data: &str) -> Function {
            let success_result = WasmEncodedResult::Success::<String> {
                value: json_data.to_string(),
                error: None,
            };
            let js_value = serde_wasm_bindgen::to_value(&success_result).unwrap();

            Function::new_no_args(&format!(
                "return {}",
                js_sys::JSON::stringify(&js_value)
                    .unwrap()
                    .as_string()
                    .unwrap()
            ))
        }

        fn create_error_callback(readable_msg: &str) -> Function {
            let error_result = WasmEncodedResult::Err::<String> {
                value: None,
                error: WasmEncodedError {
                    msg: "DatabaseError".to_string(),
                    readable_msg: readable_msg.to_string(),
                },
            };
            let js_value = serde_wasm_bindgen::to_value(&error_result).unwrap();

            Function::new_no_args(&format!(
                "return {}",
                js_sys::JSON::stringify(&js_value)
                    .unwrap()
                    .as_string()
                    .unwrap()
            ))
        }

        fn create_invalid_callback() -> Function {
            Function::new_no_args("return 42")
        }

        fn create_callback_that_throws() -> Function {
            Function::new_no_args("throw new Error('Callback error')")
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
                LocalDbQuery::execute_query_with_callback(&callback, "SELECT * FROM users").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].name, "Alice");
            assert_eq!(data[1].name, "Bob");
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_empty_success() {
            let callback = create_success_callback("[]");

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_with_callback(&callback, "SELECT * FROM empty_table")
                    .await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_database_error() {
            let callback = create_error_callback("no such table: users");

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_with_callback(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::DatabaseError { message } => {
                    assert_eq!(message, "no such table: users");
                }
                _ => panic!("Expected DatabaseError"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_invalid_json() {
            let callback = create_success_callback("{ invalid json }");

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_with_callback(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::JsonError(_) => {}
                _ => panic!("Expected JsonError"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_invalid_response_format() {
            let callback = create_invalid_callback();

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_with_callback(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::InvalidResponse => {}
                _ => panic!("Expected InvalidResponse"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_callback_throws() {
            let callback = create_callback_that_throws();

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_with_callback(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::CallbackError(_) => {}
                _ => panic!("Expected CallbackError"),
            }
        }
    }
}
