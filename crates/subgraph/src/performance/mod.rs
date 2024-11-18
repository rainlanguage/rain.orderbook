use alloy::primitives::ruint::ParseError;
use rain_orderbook_math::MathError;
use std::num::ParseIntError;
use thiserror::Error;

pub mod apy;
mod order_performance;
pub mod vol;

pub use order_performance::*;

#[derive(Error, Debug)]
pub enum PerformanceError {
    #[error(transparent)]
    MathError(#[from] MathError),
    #[error(transparent)]
    ParseUnsignedError(#[from] ParseError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}
