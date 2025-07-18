use super::*;
use alloy::{hex::FromHex, primitives::Bytes, sol_types::SolCall};
use rain_orderbook_bindings::OrderBook::multicallCall;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct MulticallCalldataResult(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(MulticallCalldataResult);

/// Generates multicall calldata from an array of individual calldatas.
///
/// Takes an array of hex-encoded calldata strings and combines them into a single
/// multicall transaction that executes all of them atomically.
///
/// ## Examples
///
/// ```javascript
/// // Get individual calldatas (withdrawals, deposits, etc.)
/// const calldatas = await Promise.all([
///   vault1.getWithdrawCalldata(amount1),
///   vault2.getWithdrawCalldata(amount2),
///   // ... more calldatas
/// ]);
///
/// // Extract values and handle errors
/// const calldata_values = calldatas.map(result => {
///   if (result.error) throw new Error(result.error.readableMsg);
///   return result.value;
/// });
///
/// // Generate multicall
/// const result = await generateMulticallCalldata(calldata_values);
/// if (result.error) {
///   console.error("Error:", result.error.readableMsg);
///   return;
/// }
/// const multicallCalldata = result.value;
/// // Use multicallCalldata for transaction
/// ```
#[wasm_export(
    js_name = "generateMulticallCalldata",
    return_description = "Multicall calldata for executing all provided calldatas atomically"
)]
pub fn generate_multicall_calldata(
    #[wasm_export(param_description = "Array of hex-encoded calldata strings to combine")]
    calldatas: Vec<String>,
) -> Result<MulticallCalldataResult, GuiError> {
    if calldatas.is_empty() {
        return Err(GuiError::InvalidCalldata(
            "No calldatas provided".to_string(),
        ));
    }

    let mut calls = Vec::new();

    for (index, calldata_str) in calldatas.iter().enumerate() {
        let calldata_bytes = Bytes::from_hex(calldata_str).map_err(|e| {
            GuiError::InvalidCalldata(format!(
                "Failed to parse calldata at index {}: {}",
                index, e
            ))
        })?;
        calls.push(calldata_bytes);
    }

    let multicall = multicallCall { data: calls };
    let encoded = multicall.abi_encode();

    Ok(MulticallCalldataResult(Bytes::from(encoded)))
}

//
// Errors
//
#[derive(Error, Debug)]
pub enum MulticallError {
    #[error("Invalid calldata: {0}")]
    InvalidCalldata(String),
}

impl MulticallError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            MulticallError::InvalidCalldata(msg) => format!("Invalid calldata: {}", msg),
        }
    }
}

impl From<GuiError> for JsValue {
    fn from(value: GuiError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<GuiError> for WasmEncodedError {
    fn from(value: GuiError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

//
// Tests
//
