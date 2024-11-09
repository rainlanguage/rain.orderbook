use alloy::primitives::{
    ruint::{FromUintError, UintTryFrom, UintTryTo},
    utils::{parse_units, ParseUnits, Unit, UnitsError},
    U256, U512,
};
use once_cell::sync::Lazy;
use thiserror::Error;

/// 1e18 or one ether in U256
pub static ONE18: Lazy<U256> = Lazy::new(|| U256::from(1_000_000_000_000_000_000_u64));

pub const FIXED_POINT_DECIMALS: u8 = 18;

#[derive(Error, Debug)]
enum MathError {
    #[error("Overflow")]
    Overflow,
    #[error(transparent)]
    UnitsError(#[from] UnitsError),
    #[error(transparent)]
    FromUintErrorU256(#[from] FromUintError<U256>),
    #[error(transparent)]
    FromUintErrorU512(#[from] FromUintError<U512>),
}

/// A trait that provide math operations as Uint256
pub trait BigUintMath {
    fn scale_up(self, scale_up_by: u8) -> Result<U256, MathError>;
    fn scale_down(self, scale_down_by: u8) -> Result<U256, MathError>;
    fn scale_18(self, decimals: u8) -> Result<U256, MathError>;
    fn mul_div(self, mul: U256, div: U256) -> Result<U256, MathError>;
    fn mul_18(self, other: U256) -> Result<U256, MathError>;
    fn div_18(self, other: U256) -> Result<U256, MathError>;
}

impl BigUintMath for U256 {
    /// Scales the value up by the given number of decimals
    fn scale_up(self, scale_up_by: u8) -> Result<U256, MathError> {
        Ok(self
            .checked_mul(UintTryTo::<U256>::uint_try_to(
                &U256::from(10_u8).pow(U256::from(scale_up_by)),
            )?)
            .ok_or(MathError::Overflow)?)
    }

    /// Scales the value down by the given number of decimals
    fn scale_down(self, scale_down_by: u8) -> Result<U256, MathError> {
        Ok(self
            .checked_div(UintTryTo::<U256>::uint_try_to(
                &U256::from(10_u8).pow(U256::from(scale_down_by)),
            )?)
            .ok_or(MathError::Overflow)?)
    }

    /// Scales the value to 18 point decimals U256
    fn scale_18(self, decimals: u8) -> Result<U256, MathError> {
        if decimals > FIXED_POINT_DECIMALS {
            self.scale_down(decimals - FIXED_POINT_DECIMALS)
        } else if decimals < FIXED_POINT_DECIMALS {
            self.scale_up(FIXED_POINT_DECIMALS - decimals)
        } else {
            Ok(self)
        }
    }

    /// Performs mulDiv operation
    fn mul_div(self, mul: U256, div: U256) -> Result<U256, MathError> {
        match self
            .widening_mul(mul)
            .checked_div(UintTryTo::<U512>::uint_try_to(&div)?)
            .ok_or(MathError::Overflow)
        {
            Ok(result) => Ok(UintTryTo::<U256>::uint_try_to(&result)?),
            Err(e) => Err(e),
        }
    }

    /// Performs 18 fixed point mul operation
    fn mul_18(self, other: U256) -> Result<U256, MathError> {
        self.mul_div(other, *ONE18)
    }

    /// Performs 18 fixed point div operation
    fn div_18(self, other: U256) -> Result<U256, MathError> {
        self.mul_div(*ONE18, other)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy::primitives::U256;
    use std::str::FromStr;

    #[test]
    fn test_to_18_decimals_happy() {
        // U256
        let value = U256::from(123456789u32);
        let result = value.scale_18(12).unwrap();
        let expected = U256::from_str("123_456_789_000_000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_18_decimals_unhappy() {
        let result = U256::from_str("4_500_000_000_000_000_000")
            .unwrap()
            .scale_18(99)
            .unwrap();
        let expected = U256::from(0);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_uint_math_scale_18() {
        // u8
        let value = U256::from(121_u8);
        let result = value.scale_18(4).unwrap();
        let expected = U256::from_str("12_100_000_000_000_000").unwrap();
        assert_eq!(result, expected);

        // U256
        let value = U256::from(123456789u32);
        let result = value.scale_18(12).unwrap();
        let expected = U256::from_str("123_456_789_000_000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_uint_math_mul_div() {
        // (10_000 * 8) / 2 = 40_000
        let value = U256::from(10_000_u16);
        let mul_value = 8.try_into().unwrap();
        let div_value = 2_u8.try_into().unwrap();
        let result = value.mul_div(mul_value, div_value).unwrap();
        let expected = U256::from(40_000_u32);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_uint_math_mul_18() {
        // 10_000 * 3 = 30_000
        let value = U256::from(10_000_u32).scale_18(0).unwrap();
        let mul_value = U256::from(3_u16).scale_18(0).unwrap();
        let result = value.mul_18(mul_value).unwrap();
        let expected = U256::from_str("30_000_000_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_uint_math_div_18() {
        // 10_000 / 3 = 3_333.3333...
        let value = U256::from(10_000_u64).scale_18(0).unwrap();
        let mul_value = U256::from(3_u8).scale_18(0).unwrap();
        let result = value.div_18(mul_value).unwrap();
        let expected = U256::from_str("3_333_333_333_333_333_333_333").unwrap();
        assert_eq!(result, expected);
    }
}
