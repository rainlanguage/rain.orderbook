use alloy_ethers_typecast::transaction::ReadableClientError;
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
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
    RpcCallError(#[from] ReadableClientError),
}
