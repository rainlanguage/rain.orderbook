use alloy::primitives::{ruint::ParseError, utils::format_units, ParseSignedError, I256, U256};
use thiserror::Error;

pub fn format_amount_u256(amount: U256, decimals: u8) -> Result<String, AmountFormatterError> {
    let formatted = format_units(amount, decimals)?;
    Ok(remove_trailing_zeros(&formatted))
}

pub fn format_amount_i256(amount: I256, decimals: u8) -> Result<String, AmountFormatterError> {
    if amount.is_negative() {
        let abs_amount = amount.abs();
        let formatted = format_units(abs_amount, decimals)?;
        Ok(format!("-{}", remove_trailing_zeros(&formatted)))
    } else {
        let formatted = format_units(amount.into_raw(), decimals)?;
        Ok(remove_trailing_zeros(&formatted))
    }
}

pub fn remove_trailing_zeros(value: &str) -> String {
    if let Some(pos) = value.find('.') {
        let integer_part = &value[..pos];
        if let Some(decimal_part) = value.get(pos + 1..) {
            let trimmed_decimal = decimal_part.trim_end_matches('0');
            if trimmed_decimal.is_empty() {
                integer_part.to_string()
            } else {
                format!("{}.{}", integer_part, trimmed_decimal)
            }
        } else {
            integer_part.to_string()
        }
    } else {
        value.to_string()
    }
}

#[derive(Error, Debug)]
pub enum AmountFormatterError {
    #[error(transparent)]
    U256ParseError(#[from] ParseError),
    #[error(transparent)]
    I256ParseError(#[from] ParseSignedError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_remove_trailing_zeros() {
        assert_eq!(remove_trailing_zeros("2.000000000000000000"), "2");
        assert_eq!(remove_trailing_zeros("-2.000000000000000000"), "-2");
        assert_eq!(remove_trailing_zeros("0.000000000000000000"), "0");
        assert_eq!(remove_trailing_zeros("123.000000000000000000"), "123");

        assert_eq!(remove_trailing_zeros("2.500000000000000000"), "2.5");
        assert_eq!(remove_trailing_zeros("-2.500000000000000000"), "-2.5");
        assert_eq!(remove_trailing_zeros("123.456000000000000000"), "123.456");
        assert_eq!(remove_trailing_zeros("0.123000000000000000"), "0.123");

        assert_eq!(remove_trailing_zeros("2.5"), "2.5");
        assert_eq!(remove_trailing_zeros("-2.5"), "-2.5");
        assert_eq!(remove_trailing_zeros("123.456"), "123.456");

        assert_eq!(remove_trailing_zeros("2"), "2");
        assert_eq!(remove_trailing_zeros("-2"), "-2");
        assert_eq!(remove_trailing_zeros("123"), "123");

        assert_eq!(remove_trailing_zeros("0."), "0");
        assert_eq!(remove_trailing_zeros("123."), "123");
        assert_eq!(remove_trailing_zeros("-123."), "-123");

        assert_eq!(remove_trailing_zeros("2.0"), "2");
        assert_eq!(remove_trailing_zeros("-2.0"), "-2");
        assert_eq!(remove_trailing_zeros("0.0"), "0");
    }

    #[test]
    fn test_format_balance_u256() {
        let balance = U256::from_str("2000000000000000000").unwrap();
        let result = format_amount_u256(balance, 18).unwrap();
        assert_eq!(result, "2");

        let balance = U256::from_str("2500000000000000000").unwrap();
        let result = format_amount_u256(balance, 18).unwrap();
        assert_eq!(result, "2.5");

        let balance = U256::from_str("1500000").unwrap();
        let result = format_amount_u256(balance, 6).unwrap();
        assert_eq!(result, "1.5");

        let balance = U256::from_str("123").unwrap();
        let result = format_amount_u256(balance, 0).unwrap();
        assert_eq!(result, "123");

        let balance = U256::ZERO;
        let result = format_amount_u256(balance, 18).unwrap();
        assert_eq!(result, "0");
    }

    #[test]
    fn test_format_balance_i256() {
        let balance = I256::from_str("2000000000000000000").unwrap();
        let result = format_amount_i256(balance, 18).unwrap();
        assert_eq!(result, "2");

        let balance = I256::from_str("2500000000000000000").unwrap();
        let result = format_amount_i256(balance, 18).unwrap();
        assert_eq!(result, "2.5");

        let balance = I256::from_str("-2000000000000000000").unwrap();
        let result = format_amount_i256(balance, 18).unwrap();
        assert_eq!(result, "-2");

        let balance = I256::from_str("-2500000000000000000").unwrap();
        let result = format_amount_i256(balance, 18).unwrap();
        assert_eq!(result, "-2.5");

        let balance = I256::from_str("-1500000").unwrap();
        let result = format_amount_i256(balance, 6).unwrap();
        assert_eq!(result, "-1.5");

        let balance = I256::ZERO;
        let result = format_amount_i256(balance, 18).unwrap();
        assert_eq!(result, "0");

        let balance = I256::from_str("1").unwrap();
        let result = format_amount_i256(balance, 18).unwrap();
        assert_eq!(result, "0.000000000000000001");

        let balance = I256::from_str("-1").unwrap();
        let result = format_amount_i256(balance, 18).unwrap();
        assert_eq!(result, "-0.000000000000000001");
    }
}
