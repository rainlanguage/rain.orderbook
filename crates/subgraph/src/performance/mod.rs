use alloy::primitives::{ruint::ParseError, U256};
use chrono::TimeDelta;
use once_cell::sync::Lazy;
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
}
