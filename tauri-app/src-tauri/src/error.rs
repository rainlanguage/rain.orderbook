use alloy::hex::FromHexError;
use alloy::primitives::ruint::{FromUintError, ParseError as FromUintParseError};
use alloy::signers::ledger::LedgerError;
use alloy_ethers_typecast::ReadableClientError;
use dotrain::error::ComposeError;
use rain_math_float::FloatError;
use rain_orderbook_app_settings::yaml::YamlError;
use rain_orderbook_common::dotrain_order::DotrainOrderError;
use rain_orderbook_common::fuzz::FuzzRunnerError;
use rain_orderbook_common::raindex_client::RaindexError;
use rain_orderbook_common::remove_order::RemoveOrderArgsError;
use rain_orderbook_common::transaction::WritableTransactionExecuteError;
use rain_orderbook_common::{
    add_order::AddOrderArgsError, csv::TryIntoCsvError, meta::TryDecodeRainlangSourceError,
    rainlang::ForkParseError, utils::timestamp::FormatTimestampDisplayError,
};
use rain_orderbook_quote::QuoteDebuggerError;
use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
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
    LedgerError(#[from] LedgerError),

    #[error(transparent)]
    TryIntoCsvError(#[from] TryIntoCsvError),

    #[error(transparent)]
    ForkParseError(#[from] ForkParseError),

    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),

    #[error(transparent)]
    TryIntoFlattenedError(#[from] FormatTimestampDisplayError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    TryDecodeRainlangSourceError(#[from] TryDecodeRainlangSourceError),

    #[error(transparent)]
    FuzzRunnerError(#[from] FuzzRunnerError),

    #[error(transparent)]
    ParseConfigYamlError(#[from] serde_yaml::Error),

    #[error(transparent)]
    RemoveOrderArgsError(#[from] RemoveOrderArgsError),

    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),

    #[error(transparent)]
    ComposeError(#[from] ComposeError),

    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),

    #[error(transparent)]
    FlattenError(#[from] rain_orderbook_common::types::FlattenError),

    #[error(transparent)]
    QuoteError(#[from] rain_orderbook_quote::error::Error),

    #[error(transparent)]
    FailedQuoteError(#[from] rain_orderbook_quote::error::FailedQuote),

    #[error(transparent)]
    FromUintParseError(#[from] FromUintParseError),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),

    #[error(transparent)]
    TradeReplayerError(#[from] rain_orderbook_common::replays::TradeReplayerError),

    #[error(transparent)]
    QuoteDebuggerError(#[from] QuoteDebuggerError),

    #[error(transparent)]
    OrderDetailError(
        #[from] rain_orderbook_subgraph_client::types::order_detail_traits::OrderDetailError,
    ),

    #[error(transparent)]
    FloatError(#[from] FloatError),

    #[error(transparent)]
    RaindexError(#[from] RaindexError),

    #[error("Missing RPCs")]
    MissingRpcs,

    #[error(transparent)]
    YamlError(#[from] YamlError),
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
