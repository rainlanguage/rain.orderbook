use crate::utils::timestamp::FormatTimestampDisplayError;
use alloy::primitives::hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlattenError {
    #[error("Error parsing BigInt: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Error parsing U256: {0}")]
    U256ParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error("Error parsing BigInt: {0}")]
    ParseSignedError(#[from] alloy::primitives::ParseSignedError),
    #[error("Error parsing BigInt: {0}")]
    FormatTimestampDisplayError(#[from] FormatTimestampDisplayError),
    #[error("Unit conversion error: {0}")]
    UnitConversionError(#[from] alloy::primitives::utils::UnitsError),
    #[error("From hex error: {0}")]
    FromHexError(#[from] FromHexError),
    #[error("ABI decode error: {0}")]
    AbiDecodeError(#[from] alloy::sol_types::Error),
    #[error("Order is missing add events, cannot determine transaction ID")]
    MissingAddEvent,
}
