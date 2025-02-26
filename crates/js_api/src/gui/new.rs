use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::prelude::*;
use wasm_function_macro::print_fn_names;

use super::GuiError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct TestStruct {
    field: String,
}
#[print_fn_names]
impl TestStruct {
    pub fn result_function() -> Result<String, GuiError> {
        Ok("Hello, world!".to_string())
    }

    pub async fn async_function() -> Result<String, GuiError> {
        // Simulate an asynchronous operation with a small delay
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            let window = web_sys::window().expect("should have a window in this context");
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve, 100, // 100ms delay
            );
        }))
        .await
        .map_err(|_| GuiError::JsError("Failed to sleep".to_string()))?;
        Ok("Hello, world!".to_string())
    }

    pub fn normal_function() -> String {
        "Hello, world!".to_string()
    }

    pub fn self_function(&self) -> String {
        self.field.clone()
    }

    fn some_private_function() -> String {
        "Hello, world!".to_string()
    }
}
