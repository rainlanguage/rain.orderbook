use alloy::primitives::{ruint::ParseError, U256};
use std::str::FromStr;
use wasm_bindgen_utils::prelude::js_sys::BigInt;

// A trait for converting types to U256
pub trait TryIntoU256 {
    type Error;
    fn try_into_u256(&self) -> Result<U256, Self::Error>;
}
impl TryIntoU256 for BigInt {
    type Error = ParseError;
    fn try_into_u256(&self) -> Result<U256, Self::Error> {
        U256::from_str(&format!("{}", &self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::ruint::BaseConvertError;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_utils::prelude::JsValue;

    #[wasm_bindgen_test]
    fn test_try_into_u256() {
        let big_int = BigInt::new(&JsValue::bigint_from_str("100")).unwrap();
        let u256 = big_int.try_into_u256().unwrap();
        assert_eq!(u256, U256::from(100));
    }

    #[wasm_bindgen_test]
    fn test_try_into_u256_zero() {
        let big_int = BigInt::new(&JsValue::bigint_from_str("0")).unwrap();
        let u256 = big_int.try_into_u256().unwrap();
        assert_eq!(u256, U256::ZERO);
    }

    #[wasm_bindgen_test]
    fn test_try_into_u256_negative() {
        let big_int = BigInt::new(&JsValue::bigint_from_str("-100")).unwrap();
        let err = big_int.try_into_u256().unwrap_err();
        assert_eq!(err, ParseError::InvalidDigit('-'));
    }

    #[wasm_bindgen_test]
    fn test_try_into_u256_max() {
        let max_u256_str = U256::MAX.to_string();
        let big_int = BigInt::new(&JsValue::bigint_from_str(&max_u256_str)).unwrap();
        let u256 = big_int.try_into_u256().unwrap();
        assert_eq!(u256, U256::MAX);
    }

    #[wasm_bindgen_test]
    fn test_try_into_u256_overflow() {
        // U256::MAX + 1, which is 2^256
        // In hex, this is "1" followed by 64 zeros.
        let overflow_val_str = format!("0x1{}", "0".repeat(64));
        let big_int = BigInt::new(&JsValue::bigint_from_str(&overflow_val_str)).unwrap();
        let err = big_int.try_into_u256().unwrap_err();
        assert_eq!(
            err,
            ParseError::BaseConvertError(BaseConvertError::Overflow)
        );
    }
}
