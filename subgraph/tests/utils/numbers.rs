use ethers::types::U256;
use std::ops::Mul;

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
