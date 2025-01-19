use alloy::primitives::{
    ruint::{FromUintError, UintTryTo},
    utils::UnitsError,
    U256, U512,
};
use std::cmp::Ordering;
use thiserror::Error;

/// 1e18 or one ether in U256
pub const ONE18: U256 = U256::from_limbs([1_000_000_000_000_000_000_u64, 0_u64, 0_u64, 0_u64]);

pub const FIXED_POINT_DECIMALS: u8 = 18;

#[derive(Error, Debug)]
pub enum MathError {
    #[error("Overflow")]
    Overflow,
    #[error(transparent)]
    UnitsError(#[from] UnitsError),
    #[error(transparent)]
    FromUintErrorU256(#[from] FromUintError<U256>),
    #[error(transparent)]
    FromUintErrorU512(Box<FromUintError<U512>>),
}
impl From<FromUintError<U512>> for MathError {
    fn from(value: FromUintError<U512>) -> Self {
        MathError::FromUintErrorU512(Box::new(value))
    }
}

/// A trait that provides math operations as Uint256
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
        self.checked_mul(UintTryTo::<U256>::uint_try_to(
            &U256::from(10_u8).pow(U256::from(scale_up_by)),
        )?)
        .ok_or(MathError::Overflow)
    }

    /// Scales the value down by the given number of decimals
    fn scale_down(self, scale_down_by: u8) -> Result<U256, MathError> {
        self.checked_div(UintTryTo::<U256>::uint_try_to(
            &U256::from(10_u8).pow(U256::from(scale_down_by)),
        )?)
        .ok_or(MathError::Overflow)
    }

    /// Scales the value to 18 point decimals U256
    fn scale_18(self, decimals: u8) -> Result<U256, MathError> {
        match decimals.cmp(&FIXED_POINT_DECIMALS) {
            Ordering::Greater => self.scale_down(decimals - FIXED_POINT_DECIMALS),
            Ordering::Less => self.scale_up(FIXED_POINT_DECIMALS - decimals),
            Ordering::Equal => Ok(self),
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
        self.mul_div(other, ONE18)
    }

    /// Performs 18 fixed point div operation
    fn div_18(self, other: U256) -> Result<U256, MathError> {
        self.mul_div(ONE18, other)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy::primitives::U256;
    use std::str::FromStr;

    #[test]
    fn test_big_uint_math_scale_18() {
        for (value, scale, expected) in &[
            // Smaller decimals, scale up
            (
                U256::from_str("123_456_789_000_000").unwrap(),
                8,
                U256::from_str("123_456_789_000_000_0000000000").unwrap(),
            ),
            (
                U256::from_str("10_000_000_000_000_000_000").unwrap(),
                8,
                U256::from_str("10_000_000_000_000_000_000_0000000000").unwrap(),
            ),
            // 18 decimals already, do nothing
            (
                U256::from_str("123_456_789_000_000_000").unwrap(),
                18,
                U256::from_str("123_456_789_000_000_000").unwrap(),
            ),
            // Larger decimals, scale down
            (
                U256::from_str("12345678900000_0000").unwrap(),
                22,
                U256::from_str("12345678900000").unwrap(),
            ),
            (
                U256::from_str("1000000000000000_0000").unwrap(),
                22,
                U256::from_str("1000000000000000").unwrap(),
            ),
            // Scaling down truncates
            (
                U256::from_str("12345678_9000000000").unwrap(),
                28,
                U256::from_str("12345678").unwrap(),
            ),
        ] {
            let result = value.scale_18(*scale).unwrap();
            assert_eq!(&result, expected);
        }
    }

    #[test]
    fn test_big_uint_math_mul_div() {
        for (value, mul_value, div_value, expected) in &[
            (
                U256::from(10_000_u16),
                8.try_into().unwrap(),
                2_u8.try_into().unwrap(),
                U256::from(40_000_u32),
            ),
            (
                U256::from_str("10_000_000_000_000_000_000").unwrap(),
                U256::from_str("8_000_000_000_000_000_000").unwrap(),
                U256::from_str("2_000_000_000_000_000_000").unwrap(),
                U256::from_str("40_000_000_000_000_000_000").unwrap(),
            ),
            // Overflows during mul
            (
                U256::from_str(
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                )
                .unwrap(),
                U256::from(2),
                U256::from(3),
                U256::from_str(
                    "77194726158210796949047323339125271902179989777093709359638389338608753093290",
                )
                .unwrap(),
            ),
        ] {
            let result = value.mul_div(*mul_value, *div_value).unwrap();
            assert_eq!(&result, expected);
        }
    }

    #[test]
    fn test_big_uint_math_mul_18() {
        for (value, mul_value, expected) in &[
            (
                U256::from(10_000_u16).scale_18(0).unwrap(),
                U256::from(3).scale_18(0).unwrap(),
                U256::from(30_000_u32).scale_18(0).unwrap(),
            ),
            // Overflows during mul
            (
                U256::from_str(
                    "0x0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                )
                .unwrap(),
                U256::from(3).scale_18(0).unwrap(),
                U256::from_str(
                    "21711016731996786641919559689128982722488122124807605757398297001483711807485",
                )
                .unwrap(),
            ),
        ] {
            let result = value.mul_18(*mul_value).unwrap();
            assert_eq!(&result, expected);
        }
    }

    #[test]
    fn test_big_uint_math_div_18() {
        for (value, mul_value, expected) in &[
            (
                U256::from(10_000_u16).scale_18(0).unwrap(),
                U256::from(3).scale_18(0).unwrap(),
                U256::from_str("3333333333333333333333").unwrap(),
            ),
            // Overflows during div
            (
                U256::from_str(
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                )
                .unwrap(),
                U256::from(3).scale_18(0).unwrap(),
                U256::from_str(
                    "38597363079105398474523661669562635951089994888546854679819194669304376546645",
                )
                .unwrap(),
            ),
        ] {
            let result = value.div_18(*mul_value).unwrap();
            assert_eq!(&result, expected);
        }
    }
}
