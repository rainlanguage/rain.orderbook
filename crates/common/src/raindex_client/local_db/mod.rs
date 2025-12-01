use crate::local_db::LocalDbError;
use wasm_bindgen_utils::prelude::*;
use wasm_bindgen_utils::result::WasmEncodedError;

pub mod executor;
pub mod pipeline;
pub mod query;

impl From<LocalDbError> for WasmEncodedError {
    fn from(value: LocalDbError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}
