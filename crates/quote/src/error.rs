use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use rain_interpreter_eval::error::ForkCallError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FailedQuote {
    #[error("Order does not exist")]
    NonExistent,
    #[error(transparent)]
    RevertError(#[from] AbiDecodedErrorType),
    #[error("Corrupt return data: {0}")]
    CorruptReturnData(String),
    #[error(transparent)]
    RevertErrorDecodeFailed(#[from] AbiDecodeFailedErrors),
    #[error(transparent)]
    ForkCallError(#[from] ForkCallError),
}
