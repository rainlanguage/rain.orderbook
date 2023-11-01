use bigdecimal::Zero;
use ethers::types::U256;
use std::{ops::Mul, str::FromStr};
extern crate bigdecimal;
use bigdecimal::BigDecimal;

pub fn get_amount_tokens(amount: u64, decimals: u8) -> U256 {
    let result: U256 = U256::from(amount).mul(U256::from(10).pow(U256::from(decimals)));

    return result;
}

pub fn display_number(number: U256, decimals: u8) -> String {
    if number.is_zero() || decimals == 0 {
        return number.to_string();
    }

    let mut result = number.to_string();
    let len = result.len() as u32;
    let decimals_u32 = decimals as u32;

    if len > decimals_u32 {
        let integer_part = &result[0..(len - decimals_u32) as usize];
        let mut decimal_part = &result[(len - decimals_u32) as usize..];

        // Remove trailing zeros from the decimal part
        decimal_part = decimal_part.trim_end_matches('0');

        if !decimal_part.is_empty() {
            result = format!("{}.{}", integer_part, decimal_part);
        } else {
            result = integer_part.to_string();
        }
    } else {
        result = format!("0.{}", "0".repeat((decimals_u32 - len) as usize));
    }

    result
}

/// TODO: WOrk in return a Result<Option<String>>
pub fn divide_decimal_strings(value_1: &str, value_2: &str) -> Option<String> {
    // Parse the input strings into BigDecimal values
    let decimal_1 = BigDecimal::from_str(value_1).expect("Invalid decimal value for value_1");
    let decimal_2 = BigDecimal::from_str(value_2).expect("Invalid decimal value for value_2");

    // Check for division by zero
    if decimal_2.is_zero() {
        return None;
    }

    // Perform the division
    let result = decimal_1 / decimal_2;

    let formatted_result = format!("{:.2}", result);

    // Format the result as a string with a specific precision
    let formatted_value = formatted_result.trim_end_matches('0').trim_end_matches('.');

    Some(formatted_value.to_string())
}
