use alloy_primitives::{ruint::ParseError, U256};

pub fn base10_str_to_u256(base10_value: &str) -> Result<U256, ParseError> {
    U256::from_str_radix(base10_value, 10)
}
