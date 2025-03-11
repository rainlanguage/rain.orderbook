use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

/// A struct that holds info of a rust error that is serializable
/// natively to JS/TS through wasm bindgen, so [Result::Err] variants
/// of binding functions can return normally in JS/TS instead of throwing.
///
/// Rust errors should impl [Into] trait to this struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WasmEncodedError {
    /// A short msg of the error
    pub msg: String,
    /// Contains the detailed human readable msg of the error
    pub readable_msg: String,
}
impl_wasm_traits!(WasmEncodedError);

/// A generic result enum that holds info of a rust [Result] that is
/// serializable natively to JS/TS through wasm bindgen, so binding
/// functions can return it normally in JS/TS instead of throwing.
///
/// Used in [wasm_bindgen_utils_macros::wasm_export!] as the returning
/// type of exporting wasm binding functions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(untagged)]
pub enum WasmEncodedResult<T> {
    /// Success variant with a value
    Success {
        value: T,
        #[tsify(type = "undefined")]
        error: Option<WasmEncodedError>,
    },
    /// Error variant with an error
    Err {
        #[tsify(type = "undefined")]
        value: Option<T>,
        error: WasmEncodedError,
    },
}

impl_wasm_traits!(WasmEncodedResult<T>);

impl<T> WasmEncodedResult<T> {
    /// Creates a success instance from the given type
    pub fn success(value: T) -> Self {
        WasmEncodedResult::Success { value, error: None }
    }
    /// Creates an error instance from the given type
    pub fn error<E: Into<WasmEncodedError>>(err: E) -> Self {
        WasmEncodedResult::Err {
            value: None,
            error: err.into(),
        }
    }
}

impl<T, E: Into<WasmEncodedError>> From<Result<T, E>> for WasmEncodedResult<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Self::success(value),
            Err(err) => Self::error(err),
        }
    }
}
