use alloy_ethers_typecast::{client::LedgerClientError, transaction::ReadableClientError};
use alloy_primitives::ruint::FromUintError;
use rain_orderbook_common::add_order::AddOrderArgsError;
use rain_orderbook_common::error::ForkParseError;
use rain_orderbook_subgraph_client::{types::flattened::TryIntoFlattenedError, OrderbookSubgraphClientError, WriteCsvError};
use serde::{ser::Serializer, Serialize};
use thiserror::Error;
use url::ParseError;

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

    #[error(transparent)]
    WriteCsvError(#[from] WriteCsvError),

    #[error(transparent)]
    ForkParseError(#[from] ForkParseError),

    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),
    
    #[error(transparent)]
    TryIntoFlattenedError(#[from] TryIntoFlattenedError),
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
