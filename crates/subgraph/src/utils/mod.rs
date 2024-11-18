use alloy::primitives::U256;
use chrono::TimeDelta;
use once_cell::sync::Lazy;
use rain_orderbook_math::{BigUintMath, MathError, ONE18};

mod order_id;
mod slice_list;

pub use order_id::*;
pub use slice_list::*;

/// a year length timestamp in seconds as 18 point decimals as U256
pub static YEAR18: Lazy<U256> =
    Lazy::new(|| U256::from(TimeDelta::days(365).num_seconds()).saturating_mul(ONE18));

/// Returns annual rate as 18 point decimals as I256
pub fn annual_rate(start: u64, end: u64) -> Result<U256, MathError> {
    U256::from(end - start)
        .saturating_mul(ONE18)
        .div_18(*YEAR18)
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_annual_rate() {
        let result = annual_rate(1, 101).unwrap();
        let expected = U256::from(3_170_979_198_376_u64);
        assert_eq!(result, expected);
    }
}
