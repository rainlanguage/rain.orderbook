use alloy::primitives::{ruint::ParseError, U256};
use chrono::TimeDelta;
use once_cell::sync::Lazy;
use rain_math_float::FloatError;
use rain_orderbook_math::{MathError, ONE18};
use std::num::ParseIntError;
use thiserror::Error;

pub mod apy;
mod order_performance;
pub mod vol;

pub use order_performance::*;

/// a year length timestamp in seconds as 18 point decimals as U256
pub static YEAR18: Lazy<U256> =
    Lazy::new(|| U256::from(TimeDelta::days(365).num_seconds()).saturating_mul(ONE18));

#[cfg(test)]
use rain_math_float::Float;
#[cfg(test)]
lazy_static::lazy_static! {
    static ref NEG7: Float = Float::parse("-7".to_string()).unwrap();
    static ref NEG2: Float = Float::parse("-2".to_string()).unwrap();
    static ref NEG1: Float = Float::parse("-1".to_string()).unwrap();
    static ref F0: Float = Float::parse("0".to_string()).unwrap();
    static ref F1: Float = Float::parse("1".to_string()).unwrap();
    static ref F2: Float = Float::parse("2".to_string()).unwrap();
    static ref F3: Float = Float::parse("3".to_string()).unwrap();
    static ref F5: Float = Float::parse("5".to_string()).unwrap();
    static ref F7: Float = Float::parse("7".to_string()).unwrap();
    static ref F10: Float = Float::parse("10".to_string()).unwrap();
    static ref F12: Float = Float::parse("12".to_string()).unwrap();
    static ref F15: Float = Float::parse("15".to_string()).unwrap();
    static ref F20: Float = Float::parse("20".to_string()).unwrap();
    static ref F25: Float = Float::parse("25".to_string()).unwrap();
    static ref F30: Float = Float::parse("30".to_string()).unwrap();
    static ref F35: Float = Float::parse("35".to_string()).unwrap();
    static ref F50: Float = Float::parse("50".to_string()).unwrap();
    static ref F100: Float = Float::parse("100".to_string()).unwrap();
    static ref FMAX: Float = Float::pack_lossless(alloy::primitives::aliases::I224::MAX, std::i32::MAX).unwrap();
}

#[cfg(test)]
fn float_hex(f: Float) -> String {
    serde_json::to_string(&f).unwrap()
}

#[derive(Error, Debug)]
pub enum PerformanceError {
    #[error(transparent)]
    MathError(#[from] MathError),
    #[error(transparent)]
    ParseUnsignedError(#[from] ParseError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("divide by zero")]
    DivByZero,
    #[error("Found no trades")]
    NoTrades,
    #[error("Float error: {0}")]
    FloatError(#[from] FloatError),
    #[error("Parsing error: {0}")]
    ParsingError(#[from] serde_json::Error),
}
