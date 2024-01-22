use alloy_ethers_typecast::{client::LedgerClientError, transaction::WritableClientError};
use thiserror::Error;

use crate::transaction::TransactionArgsError;

#[derive(Error, Debug)]
pub enum WritableTransactionExecuteError {
    #[error("WritableClient error: {0}")]
    WritableClient(#[from] WritableClientError),
    #[error("TransactionArgs error: {0}")]
    TransactionArgs(#[from] TransactionArgsError),
    #[error("LedgerClient error: {0}")]
    LedgerClient(#[from] LedgerClientError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
}
