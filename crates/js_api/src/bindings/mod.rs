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

/// Get the order hash of an order
#[wasm_export(js_name = "getOrderHash", unchecked_return_type = "string")]
pub fn get_order_hash(order: &OrderV3) -> Result<String, Error> {
    Ok(encode_prefixed(main_keccak256(order.abi_encode())))
}

/// Get takeOrders2() calldata
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

/// calculates keccak256 of the given bytes
#[wasm_export(js_name = "keccak256", unchecked_return_type = "string")]
pub fn keccak256(bytes: &[u8]) -> Result<String, Error> {
    Ok(encode_prefixed(main_keccak256(bytes)))
}

/// calculate keccak256 of a hex string
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
