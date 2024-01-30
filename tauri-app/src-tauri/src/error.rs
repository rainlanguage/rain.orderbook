use alloy_ethers_typecast::{client::LedgerClientError, transaction::ReadableClientError};
use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
use url::ParseError;
use thiserror::Error;
use serde::{Serialize, ser::Serializer};
use alloy_primitives::ruint::FromUintError;

#[derive(Debug, Error)]
pub enum CommandError {
  #[error(transparent)]
  ReadableClientError(#[from] ReadableClientError),

  #[error(transparent)]
  FromU64Error(#[from] FromUintError<u64>),

  #[error(transparent)]
  URLParseError(#[from] ParseError),

  #[error(transparent)]
  OrderbookSubgraphClientError(#[from] OrderbookSubgraphClientError),

  #[error(transparent)]
  LedgerClientError(#[from] LedgerClientError),
}

impl Serialize for CommandError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

pub type CommandResult<T> = Result<T, CommandError>;