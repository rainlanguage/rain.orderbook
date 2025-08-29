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

        let result = future
            .await
            .map_err(|e| LocalDbQueryError::PromiseError(format!("{:?}", e)))?;

        // Handle the result as an object with either error or value properties
        if let Ok(obj) = result.clone().dyn_into::<js_sys::Object>() {
            // Check for error property first
            let error_prop = js_sys::Reflect::get(&obj, &JsValue::from_str("error"));
            if let Ok(error_val) = error_prop {
                if !error_val.is_undefined() {
                    // Try to get the readableMsg from the error object
                    if let Ok(error_obj) = error_val.dyn_into::<js_sys::Object>() {
                        let readable_msg =
                            js_sys::Reflect::get(&error_obj, &JsValue::from_str("readableMsg"));
                        if let Ok(msg_val) = readable_msg {
                            if let Some(msg_str) = msg_val.as_string() {
                                return Err(LocalDbQueryError::DatabaseError { message: msg_str });
                            }
                        }
                    }

                    // Fallback to generic error message
                    return Err(LocalDbQueryError::DatabaseError {
                        message: "Database query failed".to_string(),
                    });
                }
            }

            // Check for value property
            let value_prop = js_sys::Reflect::get(&obj, &JsValue::from_str("value"));
            if let Ok(value) = value_prop {
                if let Some(json_string) = value.as_string() {
                    return serde_json::from_str(&json_string)
                        .map_err(LocalDbQueryError::JsonError);
                }
            }
        } else if let Some(json_string) = result.as_string() {
            // Fallback for direct JSON string responses
            return serde_json::from_str(&json_string).map_err(LocalDbQueryError::JsonError);
        }

        Err(LocalDbQueryError::InvalidResponse)
    }
}
