pub mod address_collectors;
pub mod decode;
pub mod fetch;
pub mod insert;
pub mod pipeline;
pub mod query;
pub mod sync;
pub mod token_fetch;

use crate::erc20::Error as TokenError;
use crate::rpc_client::RpcClientError;
use alloy::primitives::ruint::ParseError;
use alloy::primitives::{hex::FromHexError, Address};
use alloy::rpc::types::FilterBlockError;
use decode::DecodeError;
pub use fetch::{FetchConfig, FetchConfigError};
use insert::InsertError;
use query::{LocalDbQueryError, SqlBuildError};

const SUPPORTED_LOCAL_DB_CHAINS: &[u32] = &[42161];

#[derive(Debug, thiserror::Error)]
pub enum LocalDbError {
    #[error("{0}")]
    CustomError(String),

    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),

    #[error("RPC error")]
    Rpc(#[from] RpcClientError),

    #[error("JSON parsing failed")]
    JsonParse(#[from] serde_json::Error),

    #[error("Missing field: {field}")]
    MissingField { field: String },

    #[error("Invalid block number '{value}'")]
    InvalidBlockNumber {
        value: String,
        #[source]
        source: ParseError,
    },

    #[error("Events is not in expected array format")]
    InvalidEventsFormat,

    #[error("Network request timed out")]
    Timeout,

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Failed to fetch token metadata for {address} after {attempts} attempts")]
    TokenMetadataFetchFailed {
        address: Address,
        attempts: usize,
        #[source]
        source: Box<TokenError>,
    },

    #[error("Failed to check required tables")]
    TableCheckFailed(#[source] LocalDbQueryError),

    #[error("Failed to read sync status")]
    SyncStatusReadFailed(#[source] LocalDbQueryError),

    #[error("Failed to fetch events")]
    FetchEventsFailed(#[source] Box<LocalDbError>),

    #[error("Failed to decode events")]
    DecodeEventsFailed(#[source] Box<LocalDbError>),

    #[error("Failed to generate SQL from events")]
    SqlGenerationFailed(#[source] Box<LocalDbError>),

    #[error("HTTP request failed with status: {status}")]
    HttpStatus { status: u16 },

    #[error(transparent)]
    LocalDbQueryError(#[from] LocalDbQueryError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),

    #[error(transparent)]
    SqlBuildError(#[from] SqlBuildError),

    #[error("Missing topics filter")]
    MissingTopicsFilter,

    #[error("Missing block filter: {0}")]
    MissingBlockFilter(String),

    #[error("Block number is not number: {0}")]
    NonNumberBlockNumber(String),

    #[error(transparent)]
    FilterBlockError(#[from] FilterBlockError),

    #[error("Invalid retry max attempts")]
    InvalidRetryMaxAttemps,

    #[error(transparent)]
    ERC20Error(#[from] crate::erc20::Error),

    #[error(transparent)]
    FetchConfigError(#[from] FetchConfigError),

    #[error(transparent)]
    DecodeError(#[from] DecodeError),

    #[error(transparent)]
    InsertError(#[from] InsertError),
}

impl LocalDbError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            LocalDbError::CustomError(msg) => msg.clone(),
            LocalDbError::Http(err) => format!("HTTP request failed: {}", err),
            LocalDbError::Rpc(err) => format!("RPC error: {}", err),
            LocalDbError::JsonParse(err) => format!("Failed to parse JSON response: {}", err),
            LocalDbError::MissingField { field } => format!("Missing expected field: {}", field),
            LocalDbError::InvalidBlockNumber { value, .. } => {
                format!("Invalid block number provided: {}", value)
            }
            LocalDbError::InvalidEventsFormat => {
                "Events data is not in the expected array format".to_string()
            }
            LocalDbError::Timeout => "Network request timed out".to_string(),
            LocalDbError::TokenMetadataFetchFailed {
                address,
                attempts,
                source,
            } => format!(
                "Failed to fetch token metadata for {} after {} attempts: {}",
                address, attempts, source
            ),
            LocalDbError::Config { message } => format!("Configuration error: {}", message),
            LocalDbError::DecodeError(err) => format!("Event decoding error: {}", err),
            LocalDbError::InsertError(err) => {
                format!("Database insertion error: {}", err)
            }
            LocalDbError::TableCheckFailed(err) => {
                format!("Failed to check required tables: {}", err)
            }
            LocalDbError::SyncStatusReadFailed(err) => {
                format!("Failed to read sync status: {}", err)
            }
            LocalDbError::FetchEventsFailed(err) => {
                format!("Failed to fetch events: {}", err.to_readable_msg())
            }
            LocalDbError::DecodeEventsFailed(err) => {
                format!("Failed to decode events: {}", err.to_readable_msg())
            }
            LocalDbError::SqlGenerationFailed(err) => {
                format!(
                    "Failed to generate SQL from events: {}",
                    err.to_readable_msg()
                )
            }
            LocalDbError::HttpStatus { status } => {
                format!("HTTP request failed with status code: {}", status)
            }
            LocalDbError::LocalDbQueryError(err) => format!("Database query error: {}", err),
            LocalDbError::IoError(err) => format!("I/O error: {}", err),
            LocalDbError::FromHexError(err) => format!("Hex decoding error: {}", err),
            LocalDbError::SqlBuildError(err) => format!("SQL build error: {}", err),
            LocalDbError::MissingTopicsFilter => "Topics are missing from the filter".to_string(),
            LocalDbError::MissingBlockFilter(value) => {
                format!("Missing block filter: {}", value)
            }
            LocalDbError::FilterBlockError(err) => format!("Filter block error: {}", err),
            LocalDbError::NonNumberBlockNumber(value) => {
                format!("Block number is not a valid number: {}", value)
            }
            LocalDbError::InvalidRetryMaxAttemps => {
                "Invalid retry configuration for max attemps".to_string()
            }
            LocalDbError::ERC20Error(err) => format!("ERC20 error: {}", err),
            LocalDbError::FetchConfigError(err) => format!("Fetch configuration error: {}", err),
        }
    }
}

pub fn is_chain_supported_local_db(chain_id: u32) -> bool {
    SUPPORTED_LOCAL_DB_CHAINS.contains(&chain_id)
}
