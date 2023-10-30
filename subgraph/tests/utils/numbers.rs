use ethers::types::U256;
use std::ops::Mul;

pub fn get_amount_tokens(amount: u64, decimals: u8) -> U256 {
    let result: U256 = U256::from(amount).mul(U256::from(10).pow(U256::from(decimals)));

    return result;
}

pub fn format_number_with_decimals(number: U256, decimals: u8) -> String {
    let mut result = number.to_string();

    if decimals > 0 {
        if result.len() > decimals as usize {
            let integer_part = &result[..result.len() - decimals as usize];
            let decimal_part = &result[result.len() - decimals as usize..];

            result = format!("{}.{}", integer_part, decimal_part);
        } else {
            result = format!("0.{}", "0".repeat(decimals as usize - result.len()));
        }
    }

    result
}
