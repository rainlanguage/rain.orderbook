use alloy::hex::FromHexError;
use alloy::primitives::bytes::Bytes;
use alloy::sol_types::SolValue;
use alloy::{
    primitives::{
        hex::{decode, encode_prefixed},
        keccak256 as main_keccak256,
    },
    sol_types::SolCall,
};
use rain_orderbook_bindings::IOrderBookV4::{takeOrders2Call, OrderV3, TakeOrdersConfigV3};
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TakeOrdersCalldata(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(TakeOrdersCalldata);

/// Generates a unique keccak256 hash for an order, used for on-chain identification and verification.
/// This generated hash is used as the onchain order hash.
///
/// # Parameters
///
/// - `order` - Complete OrderV3 structure containing owner, evaluation logic, valid inputs/outputs, and nonce
///
/// # Returns
///
/// - `Ok(String)` - Hex-encoded hash with 0x prefix (66 characters total)
/// - `Err(Error)` - Should not occur under normal circumstances as OrderV3 is always encodable
///
/// # Examples
///
/// ```javascript
/// const result = await getOrderHash(order);
/// if (result.error) {
///   console.error("Hash generation failed:", result.error.readableMsg);
/// }
/// const orderHash = result.value;
/// // Do something with the order hash
/// ```
#[wasm_export(js_name = "getOrderHash", unchecked_return_type = "string")]
pub fn get_order_hash(order: &OrderV3) -> Result<String, Error> {
    Ok(encode_prefixed(main_keccak256(order.abi_encode())))
}

/// Generates ABI-encoded calldata for the `takeOrders2()` function on the OrderBook smart contract.
///
/// # Parameters
///
/// - `take_orders_config` - Complete configuration for order execution including:
///   - `minimumInput`: Minimum tokens to receive (wei format)
///   - `maximumInput`: Maximum tokens willing to spend (wei format)  
///   - `maximumIORatio`: Maximum acceptable price ratio for slippage control
///   - `orders`: Array of orders to execute with their specific configurations
///   - `data`: Additional arbitrary calldata for advanced use cases
///
/// # Returns
///
/// - `Ok(TakeOrdersCalldata)` - Encoded calldata ready for blockchain submission
/// - `Err(Error)` - Should not occur as TakeOrdersConfigV3 is always encodable
///
/// # Examples
///
/// ```javascript
/// const result = await getTakeOrders2Calldata(config);
/// if (result.error) {
///   console.error("Calldata generation failed:", result.error.readableMsg);
/// }
/// const calldata = result.value;
/// // Do something with the calldata
/// ```
#[wasm_export(
    js_name = "getTakeOrders2Calldata",
    unchecked_return_type = "TakeOrdersCalldata"
)]
pub fn get_take_orders2_calldata(
    take_orders_config: TakeOrdersConfigV3,
) -> Result<TakeOrdersCalldata, Error> {
    let calldata = takeOrders2Call {
        config: take_orders_config,
    }
    .abi_encode();
    Ok(TakeOrdersCalldata(Bytes::copy_from_slice(&calldata)))
}

/// Computes the keccak256 hash of raw byte data, commonly used for creating deterministic identifiers and verifying data integrity.
///
/// # Parameters
///
/// - `bytes` - Raw byte array to hash (Uint8Array in JavaScript)
///
/// # Returns
///
/// - `Ok(String)` - Hex-encoded hash with 0x prefix (66 characters total)
/// - `Err(Error)` - Should not occur as keccak256 always succeeds on valid input
///
/// # Examples
///
/// ```javascript
/// const data = new Uint8Array([1, 2, 3, 4, 5]);
/// const result = await keccak256(data);
/// if (result.error) {
///   console.error("Hash generation failed:", result.error.readableMsg);
/// }
/// const hash = result.value;
/// // Do something with the hash
/// ```
#[wasm_export(js_name = "keccak256", unchecked_return_type = "string")]
pub fn keccak256(bytes: &[u8]) -> Result<String, Error> {
    Ok(encode_prefixed(main_keccak256(bytes)))
}

/// Computes the keccak256 hash of hex-encoded string data, providing convenient hashing for blockchain-formatted data.
///
/// # Parameters
///
/// - `hex_string` - Hex-encoded string with or without 0x prefix (must contain valid hex characters)
///
/// # Returns
///
/// - `Ok(String)` - Hex-encoded hash with 0x prefix (66 characters total)
/// - `Err(Error)` - Invalid hex format, odd number of digits, or malformed input
///
/// # Examples
/// ```javascript
/// const result = await keccak256HexString("0x1234abcd");
/// if (result.error) {
///   console.error("Hash generation failed:", result.error.readableMsg);
/// }
/// const hash = result.value;
/// // Do something with the hash
/// ```
#[wasm_export(js_name = "keccak256HexString", unchecked_return_type = "string")]
pub fn keccak256_hex_string(hex_string: &str) -> Result<String, Error> {
    Ok(encode_prefixed(main_keccak256(decode(hex_string)?)))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to decode hex string")]
    FromHexError(#[from] FromHexError),
}

impl Error {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::FromHexError(e) => format!("Failed to decode hex string: {}", e),
        }
    }
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<Error> for WasmEncodedError {
    fn from(value: Error) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_get_order_hash() {
        let order = OrderV3::default();
        let result = get_order_hash(&order).unwrap();
        assert_eq!(
            result,
            "0xdcf6b886b1922d32accc60b1a0cdc53fb4bcbe74af2987b22046820030e3423b"
        );
    }

    #[wasm_bindgen_test]
    fn test_take_orders_calldata() {
        let take_orders_config = TakeOrdersConfigV3::default();
        let result = get_take_orders2_calldata(take_orders_config.clone()).unwrap();
        let expected = takeOrders2Call {
            config: take_orders_config,
        }
        .abi_encode();
        assert_eq!(result.0.to_vec(), expected);
    }

    #[wasm_bindgen_test]
    fn test_keccak256() {
        let bytes = vec![1, 2];
        let result = keccak256(&bytes).unwrap();
        let expected =
            "0x22ae6da6b482f9b1b19b0b897c3fd43884180a1c5ee361e1107a1bc635649dda".to_string();
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_keccak256_hex_string() {
        let hex_string = "0x0102";
        let result = keccak256_hex_string(&hex_string).unwrap();
        let expected =
            "0x22ae6da6b482f9b1b19b0b897c3fd43884180a1c5ee361e1107a1bc635649dda".to_string();
        assert_eq!(result, expected);

        let err = keccak256_hex_string("invalid-hex").unwrap_err();
        assert_eq!(err.to_string(), "Failed to decode hex string");
        assert_eq!(
            err.to_readable_msg(),
            "Failed to decode hex string: Odd number of digits"
        );
    }
}
