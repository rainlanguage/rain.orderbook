use alloy::primitives::{
    utils::{format_units, parse_units, ParseUnits, Unit, UnitsError},
    I256, U256,
};
use chrono::TimeDelta;

/// Returns 18 point decimals 1 as I256/U256
pub fn one_18() -> ParseUnits {
    ParseUnits::U256(U256::from(1_000_000_000_000_000_000_u64))
}

/// Returns YEAR as 18 point decimals as I256/U256
pub fn year_18() -> ParseUnits {
    ParseUnits::U256(
        U256::from(TimeDelta::days(365).num_seconds()).saturating_mul(one_18().get_absolute()),
    )
}

/// Converts a value to a 18 fixed point U256/I256 given the decimals point
pub fn scale_18_decimals<T: Into<ParseUnits>, D: TryInto<Unit, Error = UnitsError>>(
    amount: T,
    decimals: D,
) -> Result<ParseUnits, UnitsError> {
    parse_units(&format_units(amount, decimals)?, 18)
}

/// Returns annual rate as 18 point decimals as I256
pub fn annual_rate(start: u64, end: u64) -> I256 {
    I256::from_raw(U256::from(end - start).saturating_mul(one_18().get_absolute()))
        .saturating_mul(one_18().get_signed())
        .saturating_div(year_18().get_signed())
}

pub trait BigIntMath18<T: Into<ParseUnits>> {
    fn scale_18(self, decimals: u8) -> ParseUnits
    where
        Self: Sized,
        ParseUnits: From<Self>,
    {
        scale_18_decimals(self, decimals).unwrap()
    }

    fn mul_18(self, other: impl Into<ParseUnits>) -> ParseUnits
    where
        Self: Sized,
        ParseUnits: From<Self>,
    {
        let a: ParseUnits = self.into();
        let b: ParseUnits = other.into();
        if a.is_signed() || b.is_signed() {
            a.get_signed()
                .saturating_mul(b.get_signed())
                .saturating_div(one_18().get_signed())
                .into()
        } else {
            a.get_absolute()
                .saturating_mul(b.get_absolute())
                .checked_div(one_18().get_absolute())
                .unwrap_or(U256::MAX)
                .into()
        }
    }

    fn div_18(self, other: impl Into<ParseUnits>) -> ParseUnits
    where
        Self: Sized,
        ParseUnits: From<Self>,
    {
        let a: ParseUnits = self.into();
        let b: ParseUnits = other.into();
        if a.is_signed() || b.is_signed() {
            a.get_signed()
                .saturating_mul(one_18().get_signed())
                .saturating_div(b.get_signed())
                .into()
        } else {
            a.get_absolute()
                .saturating_mul(one_18().get_absolute())
                .checked_div(b.get_absolute())
                .unwrap_or(U256::MAX)
                .into()
        }
    }

    fn add_units(self, other: impl Into<ParseUnits>) -> ParseUnits
    where
        Self: Sized,
        ParseUnits: From<Self>,
    {
        let a: ParseUnits = self.into();
        let b: ParseUnits = other.into();
        if a.is_signed() || b.is_signed() {
            a.get_signed().saturating_add(b.get_signed()).into()
        } else {
            a.get_absolute().saturating_add(b.get_absolute()).into()
        }
    }

    fn sub_units(self, other: impl Into<ParseUnits>) -> ParseUnits
    where
        Self: Sized,
        ParseUnits: From<Self>,
    {
        let a: ParseUnits = self.into();
        let b: ParseUnits = other.into();
        if a.is_signed() || b.is_signed() {
            a.get_signed().saturating_sub(b.get_signed()).into()
        } else {
            a.get_absolute().saturating_sub(b.get_absolute()).into()
        }
    }
}
impl<T: Into<ParseUnits>> BigIntMath18<T> for T {}

#[cfg(test)]
mod test {
    use super::*;
    use alloy::primitives::{I256, U256};
    use std::str::FromStr;

    #[test]
    fn test_one() {
        let result = one_18();
        let expected_signed = I256::from_str("1_000_000_000_000_000_000").unwrap();
        let expected_absolute = U256::from_str("1_000_000_000_000_000_000").unwrap();
        assert_eq!(result.get_signed(), expected_signed);
        assert_eq!(result.get_absolute(), expected_absolute);
    }

    #[test]
    fn test_year_18_decimals() {
        const YEAR: u64 = 60 * 60 * 24 * 365;
        let result = year_18();
        let expected_signed = I256::try_from(YEAR)
            .unwrap()
            .saturating_mul(one_18().get_signed());
        let expected_absolute = U256::from(YEAR).saturating_mul(one_18().get_absolute());
        assert_eq!(result.get_signed(), expected_signed);
        assert_eq!(result.get_absolute(), expected_absolute);
    }

    #[test]
    fn test_to_18_decimals() {
        let value = ParseUnits::I256(I256::from_str("-123456789").unwrap());
        let result = scale_18_decimals(value, 5).unwrap();
        let expected = ParseUnits::I256(I256::from_str("-1234567890000000000000").unwrap());
        assert_eq!(result, expected);

        let value = ParseUnits::U256(U256::from_str("123456789").unwrap());
        let result = scale_18_decimals(value, 12).unwrap();
        let expected = ParseUnits::U256(U256::from_str("123456789000000").unwrap());
        assert_eq!(result, expected);
    }
}
