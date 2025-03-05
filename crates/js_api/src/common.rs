use crate::gui::GuiError;
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WasmEncodedError {
    msg: String,
    readable_msg: String,
}
impl_wasm_traits!(WasmEncodedError);

impl From<GuiError> for WasmEncodedError {
    fn from(err: GuiError) -> Self {
        WasmEncodedError {
            msg: err.to_string(),
            readable_msg: err.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WasmEncodedResult<T> {
    data: Option<T>,
    error: Option<WasmEncodedError>,
}

impl_wasm_traits!(WasmEncodedResult<()>);
impl_wasm_traits!(WasmEncodedResult<String>);
impl_wasm_traits!(WasmEncodedResult<Vec<String>>);
impl_wasm_traits!(WasmEncodedResult<bool>);

impl<T> WasmEncodedResult<T> {
    pub fn success(data: T) -> Self {
        WasmEncodedResult {
            data: Some(data),
            error: None,
        }
    }

    pub fn error(err: WasmEncodedError) -> Self {
        WasmEncodedResult {
            data: None,
            error: Some(err),
        }
    }
}

impl<T> From<Result<T, GuiError>> for WasmEncodedResult<T> {
    fn from(result: Result<T, GuiError>) -> Self {
        match result {
            Ok(data) => WasmEncodedResult {
                data: Some(data),
                error: None,
            },
            Err(err) => WasmEncodedResult {
                data: None,
                error: Some(WasmEncodedError {
                    msg: err.to_string(),
                    readable_msg: err.to_string(),
                }),
            },
        }
    }
}
