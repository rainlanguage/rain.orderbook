use std::str::FromStr;

// A trait for converting types to U256
pub trait TryIntoU256 {
    type Error;
    fn try_into_u256(&self) -> Result<alloy::primitives::U256, Self::Error>;
}
impl TryIntoU256 for wasm_bindgen_utils::prelude::js_sys::BigInt {
    type Error = alloy::primitives::ruint::ParseError;
    fn try_into_u256(&self) -> Result<alloy::primitives::U256, Self::Error> {
        alloy::primitives::U256::from_str(&format!("{}", &self))
    }
}
