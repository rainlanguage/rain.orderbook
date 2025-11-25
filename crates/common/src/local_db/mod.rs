pub mod address_collectors;
pub mod decode;
pub mod export;
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
use rain_orderbook_app_settings::remote::manifest::FetchManifestError;
use rain_orderbook_app_settings::yaml::YamlError;
use std::num::ParseIntError;
use strict_yaml_rust::ScanError;
use tokio::task::JoinError;

const SUPPORTED_LOCAL_DB_CHAINS: &[u32] = &[42161];

#[derive(Debug, thiserror::Error)]
pub enum LocalDbError {
    #[error("{0}")]
    CustomError(String),

    #[error(transparent)]
    SettingsYaml(#[from] YamlError),

    #[error(transparent)]
    YamlScan(#[from] ScanError),

    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),

    #[error("RPC error")]
    Rpc(#[from] RpcClientError),

    #[error("JSON parsing failed")]
    JsonParse(#[from] serde_json::Error),

    #[error("Missing field: {field}")]
    MissingField { field: String },

    #[error("Missing local-db sync config for network '{network}'")]
    MissingLocalDbSyncForNetwork { network: String },

    #[error("Missing local-db remote for orderbook '{orderbook_key}'")]
    MissingLocalDbRemote { orderbook_key: String },

    #[error("Invalid block number '{value}'")]
    InvalidBlockNumber {
        value: String,
        #[source]
        source: ParseError,
    },
    #[error("Invalid block number '{value}'")]
    InvalidBlockNumberString {
        value: String,
        #[source]
        source: ParseIntError,
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
    ManifestFetch(#[from] FetchManifestError),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] JoinError),

    #[error(transparent)]
    LocalDbQueryError(#[from] LocalDbQueryError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),

    #[error(transparent)]
    SqlBuildError(#[from] SqlBuildError),

    #[error(transparent)]
    InsertError(#[from] InsertError),

    #[error("Overflow when incrementing last_synced_block: {last_synced_block}")]
    LastSyncedBlockOverflow { last_synced_block: u64 },
    #[error("There are no rows in the db_metadata table")]
    MissingDbMetadataRow,

    #[error("Database schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: u32, found: u32 },

    #[error("Invalid bootstrap implementation")]
    InvalidBootstrapImplementation,

    #[error("Block sync threshold exceeded: latest block {latest_block}, last indexed block {last_indexed_block}, threshold {threshold}")]
    BlockSyncThresholdExceeded {
        latest_block: u64,
        last_indexed_block: u64,
        threshold: u64,
    },

    #[error("Invalid log index '{value}'")]
    InvalidLogIndex {
        value: String,
        #[source]
        source: ParseIntError,
    },

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
    Export(#[from] export::ExportError),

    #[error(transparent)]
    DecodeError(#[from] DecodeError),
}

impl LocalDbError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            LocalDbError::CustomError(msg) => msg.clone(),
            LocalDbError::SettingsYaml(err) => format!("Settings parsing failed: {}", err),
            LocalDbError::YamlScan(err) => format!("Settings YAML scan failed: {}", err),
            LocalDbError::Http(err) => format!("HTTP request failed: {}", err),
            LocalDbError::Rpc(err) => format!("RPC error: {}", err),
            LocalDbError::JsonParse(err) => format!("Failed to parse JSON response: {}", err),
            LocalDbError::MissingField { field } => format!("Missing expected field: {}", field),
            LocalDbError::MissingLocalDbRemote { orderbook_key } => format!(
                "Missing local-db remote configuration for orderbook '{}'",
                orderbook_key
            ),
            LocalDbError::InvalidBlockNumber { value, .. } => {
                format!("Invalid block number provided: {}", value)
            }
            LocalDbError::InvalidBlockNumberString { value, .. } => {
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
            LocalDbError::MissingLocalDbSyncForNetwork { network } => format!(
                "Missing local-db sync configuration for network '{}'",
                network
            ),
            LocalDbError::Config { message } => format!("Configuration error: {}", message),
            LocalDbError::DecodeError(err) => format!("Event decoding error: {}", err),
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
            LocalDbError::ManifestFetch(err) => format!("Failed to fetch manifest: {}", err),
            LocalDbError::TaskJoin(err) => format!("Task join error: {}", err),
            LocalDbError::LocalDbQueryError(err) => format!("Database query error: {}", err),
            LocalDbError::IoError(err) => format!("I/O error: {}", err),
            LocalDbError::FromHexError(err) => format!("Hex decoding error: {}", err),
            LocalDbError::SqlBuildError(err) => format!("SQL build error: {}", err),
            LocalDbError::InsertError(err) => format!("Data insertion error: {}", err),
            LocalDbError::LastSyncedBlockOverflow { last_synced_block } => format!(
                "Overflow when incrementing last_synced_block {}",
                last_synced_block
            ),
            LocalDbError::MissingDbMetadataRow => {
                "There are no rows in the db_metadata table".to_string()
            }
            LocalDbError::SchemaVersionMismatch { expected, found } => format!(
                "Database schema version mismatch: expected {}, found {}",
                expected, found
            ),
            LocalDbError::InvalidBootstrapImplementation => {
                "This bootstrap implementation is invalid.".to_string()
            }
            LocalDbError::BlockSyncThresholdExceeded {
                latest_block,
                last_indexed_block,
                threshold,
            } => format!(
                "Block sync threshold exceeded: latest block {}, last indexed block {}, threshold {}",
                latest_block, last_indexed_block, threshold
            ),
            LocalDbError::InvalidLogIndex { value, .. } => {
                format!("Invalid log index provided: {}", value)
            }
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
            LocalDbError::Export(err) => format!("Export error: {}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderbookIdentifier {
    pub chain_id: u32,
    pub orderbook_address: Address,
}
impl OrderbookIdentifier {
    pub fn new(chain_id: u32, orderbook_address: Address) -> Self {
        Self {
            chain_id,
            orderbook_address,
        }
    }
}

pub fn is_chain_supported_local_db(chain_id: u32) -> bool {
    SUPPORTED_LOCAL_DB_CHAINS.contains(&chain_id)
}
