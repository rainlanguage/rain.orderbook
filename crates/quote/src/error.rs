use alloy::primitives::hex::FromHexError;
use alloy_ethers_typecast::transaction::ReadableClientError;
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
use thiserror::Error;
use url::ParseError;

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
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    RpcCallError(#[from] ReadableClientError),
    #[error(transparent)]
    UrlParseError(#[from] ParseError),
    #[error(transparent)]
    SubgraphClientError(#[from] OrderbookSubgraphClientError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    AlloySolTypesError(#[from] alloy::sol_types::Error),
}
