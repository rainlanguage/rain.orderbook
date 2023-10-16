use ethers::types::U256;
use std::ops::Mul;

pub fn _get_amount_tokens(amount: u64, decimals: u8) -> U256 {
    let result: U256 = U256::from(amount).mul(U256::from(10).pow(U256::from(decimals)));

    return result;
}
