use crate::{cynic_client::CynicClientError, pagination::PaginationClientError};
use alloy::primitives::{
    ruint::ParseError, utils::UnitsError, BigIntConversionError, ParseSignedError,
};
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderbookSubgraphClientError {
    #[error(transparent)]
    CynicClientError(#[from] CynicClientError),
    #[error("Subgraph query returned no data")]
    Empty,
    #[error(transparent)]
    PaginationClientError(#[from] PaginationClientError),
    #[error(transparent)]
    ParseNumberError(#[from] crate::error::ParseNumberError),
}

#[derive(Error, Debug)]
pub enum ParseNumberError {
    #[error(transparent)]
    UnitsError(#[from] UnitsError),
    #[error(transparent)]
    ParseUnsignedError(#[from] ParseError),
    #[error(transparent)]
    ParseSignedError(#[from] ParseSignedError),
    #[error(transparent)]
    BigIntConversionError(#[from] BigIntConversionError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

impl From<UnitsError> for OrderbookSubgraphClientError {
    fn from(value: UnitsError) -> Self {
        ParseNumberError::from(value).into()
    }
}
impl From<ParseError> for OrderbookSubgraphClientError {
    fn from(value: ParseError) -> Self {
        ParseNumberError::from(value).into()
    }
}
impl From<ParseSignedError> for OrderbookSubgraphClientError {
    fn from(value: ParseSignedError) -> Self {
        ParseNumberError::from(value).into()
    }
}
impl From<BigIntConversionError> for OrderbookSubgraphClientError {
    fn from(value: BigIntConversionError) -> Self {
        ParseNumberError::from(value).into()
    }
}
impl From<ParseIntError> for OrderbookSubgraphClientError {
    fn from(value: ParseIntError) -> Self {
        ParseNumberError::from(value).into()
    }
}
impl From<ParseFloatError> for OrderbookSubgraphClientError {
    fn from(value: ParseFloatError) -> Self {
        ParseNumberError::from(value).into()
    }
}
