use crate::transaction::TransactionArgsError;
use alloy_ethers_typecast::{client::LedgerClientError, transaction::WritableClientError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WritableTransactionExecuteError {
    #[error(transparent)]
    WritableClient(#[from] WritableClientError),
    #[error(transparent)]
    TransactionArgs(#[from] TransactionArgsError),
    #[error(transparent)]
    LedgerClient(#[from] LedgerClientError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
}
