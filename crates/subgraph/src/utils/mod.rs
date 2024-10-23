use alloy::primitives::utils::{format_units, parse_units, ParseUnits, Unit, UnitsError};
use chrono::TimeDelta;

mod order_id;
mod slice_list;

pub use order_id::*;
pub use slice_list::*;

/// Returns 18 point decimals 1 as I256/U256
pub fn one_18() -> ParseUnits {
    parse_units("1", 18).unwrap()
}

/// Returns YEAR as 18 point decimals as I256/U256
pub fn year_18() -> ParseUnits {
    parse_units(&TimeDelta::days(365).num_seconds().to_string(), 18).unwrap()
}

/// Converts a U256/I256 value to a 18 fixed point U256/I256 given the decimals point
pub fn to_18_decimals<T: TryInto<Unit, Error = UnitsError>>(
    amount: ParseUnits,
    decimals: T,
) -> Result<ParseUnits, UnitsError> {
    parse_units(&format_units(amount, decimals)?, 18)
}

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
        let result = to_18_decimals(value, 5).unwrap();
        let expected = ParseUnits::I256(I256::from_str("-1234567890000000000000").unwrap());
        assert_eq!(result, expected);

        let value = ParseUnits::U256(U256::from_str("123456789").unwrap());
        let result = to_18_decimals(value, 12).unwrap();
        let expected = ParseUnits::U256(U256::from_str("123456789000000").unwrap());
        assert_eq!(result, expected);
    }
}
