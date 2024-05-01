use crate::error::CommandResult;
use rain_interpreter_eval::error::{AbiDecodedErrorType, selector_registry_abi_decode};
use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize)]
#[serde(tag = "type", content = "content")]
pub enum ExtAbiDecodedErrorType {
    Unknown(Vec<u8>),
    Known {
        name: String,
        args: Vec<String>,
        sig: String,
    },
}

#[tauri::command]
pub async fn decode_errors(error_data: Vec<u8>) -> CommandResult<AbiDecodedErrorType> {
    let decoded_error = selector_registry_abi_decode(error_data.as_slice()).await?;
    Ok(decoded_error)
}
