use super::*;
use alloy::primitives::U256;
use js_sys::Uint8Array;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;
use thiserror::Error;
use wasm_bindgen::{prelude::*, JsValue};

// a serializer fn for serializing U256 to bytes
fn u256_to_bytes_serilializer<S: Serializer>(val: &U256, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(&val.to_be_bytes_trimmed_vec())
}

// a deserializer fn for deserializing bytes to U256, saturates at U256::MAX
fn u256_from_bytes_deserializer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<U256, D::Error> {
    Ok(
        U256::try_from_be_slice(ByteBuf::deserialize(deserializer)?.as_slice())
            .unwrap_or(U256::MAX),
    )
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    UnitsError(#[from] UnitsError),
}
impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

/// BigUint is class object that provides math operations functionalities for
/// Uint256 as Uint8Array in big endian format
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct BigUint {
    #[serde(serialize_with = "u256_to_bytes_serilializer")]
    #[serde(deserialize_with = "u256_from_bytes_deserializer")]
    value: U256,
}

#[wasm_bindgen]
impl BigUint {
    /// Create a new instance of BigUint, saturates at `Uint256.MAX`
    /// if the given value is greater than Uint256 range
    #[wasm_bindgen(constructor)]
    pub fn new(value: &[u8]) -> BigUint {
        BigUint {
            value: U256::try_from_be_slice(value).unwrap_or(U256::MAX),
        }
    }

    /// This instance's value
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Uint8Array {
        self.value.to_be_bytes_trimmed_vec().as_slice().into()
    }

    // setter for this struct instance's value field
    #[wasm_bindgen(setter)]
    pub fn set_value(&mut self, value: &[u8]) {
        self.value = U256::try_from_be_slice(value).unwrap_or(U256::MAX);
    }

    /// Scales this instance to 18 point decimals given a decimal point
    #[wasm_bindgen(js_name = scale18)]
    pub fn scale_18(&self, decimals: u8) -> Result<BigUint, Error> {
        Ok(self
            .value
            .scale_18(decimals)
            .map(|v| BigUint::new(&v.to_be_bytes_trimmed_vec()))?)
    }

    /// Performs mulDiv operation
    #[wasm_bindgen(js_name = mulDiv)]
    pub fn mul_div(&self, mul: &[u8], div: &[u8]) -> BigUint {
        BigUint::new(
            self.value
                .mul_div(
                    U256::try_from_be_slice(mul).unwrap_or(U256::MAX),
                    U256::try_from_be_slice(div).unwrap_or(U256::MAX),
                )
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        )
    }

    /// Performs 18 fixed point mul operation
    #[wasm_bindgen(js_name = mul18)]
    pub fn mul_18(&self, other: &[u8]) -> BigUint {
        BigUint::new(
            self.value
                .mul_18(U256::try_from_be_slice(other).unwrap_or(U256::MAX))
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        )
    }

    /// Performs 18 fixed point div operation
    #[wasm_bindgen(js_name = div18)]
    pub fn div_18(&self, other: &[u8]) -> BigUint {
        BigUint::new(
            self.value
                .div_18(U256::try_from_be_slice(other).unwrap_or(U256::MAX))
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_new() {
        let result = BigUint::new(&[1u8, 0u8]);
        let expected = BigUint {
            value: U256::from(256),
        };
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_new_saturated() {
        let result = BigUint::new(&[1u8; 60]);
        let expected = BigUint { value: U256::MAX };
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_scale_18_happy() {
        let result = BigUint::new(&[255]).scale_18(2).unwrap();
        let expected = BigUint::new(
            U256::from_str("2_550_000_000_000_000_000")
                .unwrap()
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        );
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_scale_18_unhappy() {
        let result = BigUint::new(&[255]).scale_18(99);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_mul_div() {
        let result = BigUint::new(&[255]).scale_18(2).unwrap();
        let expected = BigUint::new(
            U256::from_str("2_550_000_000_000_000_000")
                .unwrap()
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        );
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_mul_18() {
        let result = BigUint::new(&[255]).scale_18(2).unwrap().mul_18(
            10u8.scale_18(0)
                .unwrap()
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        );
        let expected = BigUint::new(
            U256::from_str("25_500_000_000_000_000_000")
                .unwrap()
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        );
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_div_18() {
        let result = BigUint::new(&[255]).scale_18(2).unwrap().div_18(
            10u8.scale_18(0)
                .unwrap()
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        );
        let expected = BigUint::new(
            U256::from_str("255_000_000_000_000_000")
                .unwrap()
                .to_be_bytes_trimmed_vec()
                .as_slice(),
        );
        assert_eq!(result, expected);
    }
}
