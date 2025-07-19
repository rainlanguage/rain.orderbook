use alloy::{hex::FromHex, primitives::Bytes, sol_types::SolCall};
use rain_orderbook_bindings::OrderBook::multicallCall;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

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
) -> Result<MulticallCalldataResult, MulticallError> {
    if calldatas.is_empty() {
        return Err(MulticallError::InvalidCalldata(
            "No calldatas provided".to_string(),
        ));
    }

    let mut calls = Vec::new();

    for (index, calldata_str) in calldatas.iter().enumerate() {
        let hex_part = calldata_str.strip_prefix("0x").unwrap_or(calldata_str);
        if hex_part.is_empty() {
            return Err(MulticallError::InvalidCalldata(format!(
                "Failed to parse calldata at index {}: empty string",
                index
            )));
        }
        if hex_part.len() % 2 != 0 {
            return Err(MulticallError::InvalidCalldata(format!(
                "Failed to parse calldata at index {}: odd length hex",
                index
            )));
        }
        let calldata_bytes = Bytes::from_hex(calldata_str).map_err(|e| {
            MulticallError::InvalidCalldata(format!(
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
#[derive(Error, Debug, Clone)]
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

impl From<MulticallError> for JsValue {
    fn from(value: MulticallError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<MulticallError> for WasmEncodedError {
    fn from(value: MulticallError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

//
// Tests
//
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_success() {
        // Test with valid calldata
        let calldatas = vec![
            "0x1234567890abcdef".to_string(),
            "0xdeadbeef".to_string(),
            "0x12345678".to_string(),
        ];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_ok());

        let multicall_result = result.unwrap();
        // Verify the result contains encoded multicall data
        assert!(!multicall_result.0.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_single_call() {
        // Test with single calldata
        let calldatas = vec!["0x1234567890abcdef".to_string()];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_ok());

        let multicall_result = result.unwrap();
        assert!(!multicall_result.0.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_empty_input() {
        // Test with empty input
        let calldatas: Vec<String> = vec![];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_invalid_hex() {
        // Test with invalid hex string
        let calldatas = vec!["0x1234567890abcdef".to_string(), "invalid_hex".to_string()];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_odd_length_hex() {
        // Test with odd length hex string
        let calldatas = vec![
            "0x1234567890abcdef".to_string(),
            "0xdeadbeef1".to_string(), // odd length
        ];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_empty_string() {
        // Test with empty string in calldata
        let calldatas = vec!["0x1234567890abcdef".to_string(), "".to_string()];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_just_0x() {
        // Test with just "0x"
        let calldatas = vec!["0x1234567890abcdef".to_string(), "0x".to_string()];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_generate_multicall_calldata_real_world_example() {
        // Test with realistic calldata examples
        let calldatas = vec![
            // Example ERC20 transfer calldata
            "0xa9059cbb000000000000000000000000742b6c4e4e4e4e4e4e4e4e4e4e4e4e4e4e4e4e0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
            // Example approve calldata
            "0x095ea7b3000000000000000000000000742b6c4e4e4e4e4e4e4e4e4e4e4e4e4e4e4e0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
        ];

        let result = generate_multicall_calldata(calldatas);
        assert!(result.is_ok());

        let multicall_result = result.unwrap();
        assert!(!multicall_result.0.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_multicall_error_to_readable_msg() {
        let error = MulticallError::InvalidCalldata("test error".to_string());
        assert_eq!(error.to_readable_msg(), "Invalid calldata: test error");
    }
}
