use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use wasm_function_macro::impl_wasm_exports;

use super::GuiError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct CustomError {
    msg: String,
    readable_msg: String,
}
impl_wasm_traits!(CustomError);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct CustomResult<T> {
    data: Option<T>,
    error: Option<CustomError>,
}
impl_wasm_traits!(CustomResult<String>);

impl<T> From<Result<T, GuiError>> for CustomResult<T> {
    fn from(result: Result<T, GuiError>) -> Self {
        match result {
            Ok(data) => CustomResult {
                data: Some(data),
                error: None,
            },
            Err(err) => CustomResult {
                data: None,
                error: Some(CustomError {
                    msg: err.to_string(),
                    readable_msg: err.to_string(),
                }),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct TestStruct {
    field: String,
}
#[impl_wasm_exports]
impl TestStruct {
    pub fn new(value: String) -> Self {
        Self { field: value }
    }

    pub fn simple_function() -> Result<String, GuiError> {
        Ok("Hello, world!".to_string())
    }

    pub fn err_function() -> Result<String, GuiError> {
        Err(GuiError::JsError("some error".to_string()))
    }

    pub fn simple_function_with_self(&self) -> Result<String, GuiError> {
        Ok(format!("Hello, {}!", self.field))
    }

    pub fn err_function_with_self(&self) -> Result<String, GuiError> {
        Err(GuiError::JsError("some error".to_string()))
    }

    // pub async fn async_function() -> Result<String, GuiError> {
    //     // Simulate an asynchronous operation with a small delay
    //     wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
    //         let window = web_sys::window().expect("should have a window in this context");
    //         let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
    //             &resolve, 100, // 100ms delay
    //         );
    //     }))
    //     .await
    //     .map_err(|_| GuiError::JsError("Failed to sleep".to_string()))?;
    //     Ok("Hello, world!".to_string())
    // }

    // pub fn normal_function() -> String {
    //     "Hello, world!".to_string()
    // }

    // pub fn self_function(&self) -> String {
    //     self.field.clone()
    // }

    // fn some_private_function() -> String {
    //     "Hello, world!".to_string()
    // }
}
