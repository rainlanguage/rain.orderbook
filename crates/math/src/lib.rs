use alloy::primitives::{
    ruint::UintTryFrom,
    utils::{parse_units, ParseUnits, Unit, UnitsError},
    U256,
};

#[cfg(target_family = "wasm")]
pub mod js_api;

/// Returns 18 point decimals 1 as U256
pub fn one_18() -> U256 {
    U256::from(1_000_000_000_000_000_000_u64)
}

/// Converts a value to a 18 fixed point U256 given the decimals point
/// A valid ethereum decimals point range is always less than 77
pub fn scale_18_decimals<T, D: TryInto<Unit, Error = UnitsError>>(
    amount: T,
    decimals: D,
) -> Result<U256, UnitsError>
where
    U256: UintTryFrom<T>,
{
    Ok(parse_units(
        &ParseUnits::U256(U256::from(amount)).format_units(decimals.try_into()?),
        18,
    )?
    .get_absolute())
}

/// A trait that provide math operations for big integers
pub trait BigUintMath<T> {
    /// Scales the value to 18 point decimals U256
    fn scale_18<D: TryInto<Unit, Error = UnitsError>>(self, decimals: D) -> Result<U256, UnitsError>
    where
        Self: Sized,
        U256: UintTryFrom<Self>,
    {
        scale_18_decimals(self, decimals)
    }

    /// mulDiv operation
    fn mul_div<K, U>(self, mul: K, div: U) -> U256
    where
        Self: Sized,
        K: Sized,
        U: Sized,
        U256: UintTryFrom<Self> + UintTryFrom<K> + UintTryFrom<U>,
    {
        U256::from(self)
            .saturating_mul(U256::from(mul))
            .checked_div(U256::from(div))
            .unwrap_or(U256::MAX)
    }

    /// 18 fixed point mul operation
    fn mul_18<K>(self, other: K) -> U256
    where
        Self: Sized,
        K: Sized,
        U256: UintTryFrom<Self> + UintTryFrom<K>,
    {
        U256::from(self).mul_div(other, one_18())
    }

    /// 18 fixed point div operation
    fn div_18<K>(self, other: K) -> U256
    where
        Self: Sized,
        K: Sized,
        U256: UintTryFrom<Self> + UintTryFrom<K>,
    {
        U256::from(self).mul_div(one_18(), other)
    }
}
impl<T> BigUintMath<T> for T where U256: UintTryFrom<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use alloy::primitives::U256;
    use std::str::FromStr;

    #[test]
    fn test_one() {
        let result = one_18();
        let expected = U256::from_str("1_000_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_18_decimals_happy() {
        // u8
        let value = 45_u8;
        let result = scale_18_decimals(value, 1).unwrap();
        let expected = U256::from_str("4_500_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);

        // u32
        let value = 123456789_u32;
        let result = scale_18_decimals(value, 12).unwrap();
        let expected = U256::from_str("123_456_789_000_000").unwrap();
        assert_eq!(result, expected);

        // U256
        let value = U256::from(123456789u32);
        let result = scale_18_decimals(value, 12).unwrap();
        let expected = U256::from_str("123_456_789_000_000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_18_decimals_unhappy() {
        let result = scale_18_decimals(
            U256::from_str("4_500_000_000_000_000_000").unwrap(),
            99, // invalid ethereum decimals unit
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_big_uint_math_scale_18() {
        // u8
        let value = 121_u8;
        let result = value.scale_18(4).unwrap();
        let expected = U256::from_str("12_100_000_000_000_000").unwrap();
        assert_eq!(result, expected);

        // u32
        let value = 123456789_u32;
        let result = value.scale_18(12).unwrap();
        let expected = U256::from_str("123_456_789_000_000").unwrap();
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
        let value = 10_000_u16;
        let mul_value = U256::from(8);
        let div_value = 2_u8;
        let result = value.mul_div(mul_value, div_value);
        let expected = U256::from(40_000_u32);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_uint_math_mul_18() {
        // 10_000 * 3 = 30_000
        let value = 10_000_u32.scale_18(0).unwrap();
        let mul_value = 3_u16.scale_18(0).unwrap();
        let result = value.mul_18(mul_value);
        let expected = U256::from_str("30_000_000_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_uint_math_div_18() {
        // 10_000 / 3 = 3_333.3333...
        let value = 10_000_u64.scale_18(0).unwrap();
        let mul_value = 3_u8.scale_18(0).unwrap();
        let result = value.div_18(mul_value);
        let expected = U256::from_str("3_333_333_333_333_333_333_333").unwrap();
        assert_eq!(result, expected);
    }
}
