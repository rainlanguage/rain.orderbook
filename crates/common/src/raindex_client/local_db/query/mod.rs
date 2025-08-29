use thiserror::Error;
use wasm_bindgen_utils::prelude::{
    js_sys, wasm_bindgen, wasm_bindgen_futures::JsFuture, JsCast, JsValue,
};

const TABLE_SCHEMA_SQL: &str = include_str!("../sql/tables.sql");

#[derive(Error, Debug)]
pub enum LocalDbError {
    #[error("Database query failed: {0}")]
    CustomError(String),
}

/// Execute a query that returns data and deserialize the result into the specified type
async fn execute_query_with_callback<T>(
    callback: &js_sys::Function,
    sql: &str,
) -> Result<T, LocalDbError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let result = callback
        .call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str(sql),
        )
        .map_err(|e| LocalDbError::CustomError(format!("Callback error: {:?}", e)))?;

    let promise = js_sys::Promise::resolve(&result);
    let future = JsFuture::from(promise);

    let result = future
        .await
        .map_err(|e| LocalDbError::CustomError(format!("Promise error: {:?}", e)))?;

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
                            return Err(LocalDbError::CustomError(msg_str));
                        }
                    }
                }

                // Fallback to generic error message
                return Err(LocalDbError::CustomError(
                    "Database query failed".to_string(),
                ));
            }
        }

        // Check for value property
        let value_prop = js_sys::Reflect::get(&obj, &JsValue::from_str("value"));
        if let Ok(value) = value_prop {
            if let Some(json_string) = value.as_string() {
                return serde_json::from_str(&json_string).map_err(|e| {
                    LocalDbError::CustomError(format!("JSON deserialization error: {:?}", e))
                });
            }
        }
    } else if let Some(json_string) = result.as_string() {
        // Fallback for direct JSON string responses
        return serde_json::from_str(&json_string).map_err(|e| {
            LocalDbError::CustomError(format!("JSON deserialization error: {:?}", e))
        });
    }

    Err(LocalDbError::CustomError(
        "No valid response from database query".to_string(),
    ))
}
