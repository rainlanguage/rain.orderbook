use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// reqwest::Error, FromHex are not cloneable and serializeable, hence need to
// convert it to stringly typed, those errors also dont have nested types, so
// it doesnt introduce any meaningfull difference
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorDecodeFailed {
    #[error("Reqwest error: {0}")]
    ReqwestError(String),
    #[error("Invalid Error Selector: {0:?}")]
    InvalidSelectorHash(Vec<u8>),
    #[error("Selectors Cache Poisoned")]
    SelectorsCachePoisoned,
    #[error("FromHex error: {0}")]
    HexDecodeError(String),
    #[error("No Error Data")]
    NoData,
}

impl From<AbiDecodeFailedErrors> for ErrorDecodeFailed {
    fn from(value: AbiDecodeFailedErrors) -> Self {
        match value {
            AbiDecodeFailedErrors::NoData => Self::NoData,
            AbiDecodeFailedErrors::SelectorsCachePoisoned => Self::SelectorsCachePoisoned,
            AbiDecodeFailedErrors::InvalidSelectorHash(e) => Self::InvalidSelectorHash(e),
            AbiDecodeFailedErrors::ReqwestError(e) => Self::ReqwestError(e.to_string()),
            AbiDecodeFailedErrors::HexDecodeError(e) => Self::HexDecodeError(e.to_string()),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailedQuote {
    #[error("Order does not exist")]
    NonExistent,
    #[error("{0}")]
    CorruptReturnedData(String),
    #[error(transparent)]
    AbiDecodedRevertError(#[from] AbiDecodedErrorType),
    #[error(transparent)]
    ErrorDecodeFailed(#[from] ErrorDecodeFailed),
}

impl From<AbiDecodeFailedErrors> for FailedQuote {
    fn from(value: AbiDecodeFailedErrors) -> Self {
        Self::ErrorDecodeFailed(value.into())
    }
}
