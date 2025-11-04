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
use crate::rpc_client::{LogEntryResponse, RpcClient, RpcClientError};
use alloy::primitives::ruint::ParseError;
use alloy::primitives::{hex::FromHexError, Address};
use alloy::rpc::types::FilterBlockError;
use decode::{decode_events as decode_events_impl, DecodedEvent, DecodedEventData};
pub use fetch::{FetchConfig, FetchConfigError};
use insert::{
    decoded_events_to_statements as decoded_events_to_statements_impl,
    raw_events_to_statements as raw_events_to_statements_impl, InsertError,
};
use query::{LocalDbQueryError, SqlBuildError, SqlStatementBatch};
use rain_orderbook_app_settings::remote::manifest::FetchManifestError;
use rain_orderbook_app_settings::yaml::YamlError;
use std::collections::HashMap;
use std::num::ParseIntError;
use strict_yaml_rust::ScanError;
use tokio::task::JoinError;
use url::Url;
use wasm_bindgen_utils::prelude::*;

const SUPPORTED_LOCAL_DB_CHAINS: &[u32] = &[42161];

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LocalDb {
    rpc_client: RpcClient,
}

#[cfg(test)]
impl Default for LocalDb {
    fn default() -> Self {
        let url = Url::parse("foo://example.com").unwrap();
        let rpc_client = RpcClient::new_with_urls(vec![url]).unwrap();
        Self { rpc_client }
    }
}

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

    #[error("Missing runner target for chain {chain_id} orderbook {orderbook_address}")]
    MissingRunnerTarget {
        chain_id: u32,
        orderbook_address: Address,
    },

    #[error(
        "Network '{network_key}' has mismatched chain ids (expected {expected}, found {found})"
    )]
    RunnerNetworkChainIdMismatch {
        network_key: String,
        expected: u32,
        found: u32,
    },

    #[error("Failed to build dump url '{url}'")]
    DumpUrlConstructionFailed {
        url: String,
        #[source]
        source: url::ParseError,
    },

    #[error("Failed to fetch token metadata for {address} after {attempts} attempts")]
    TokenMetadataFetchFailed {
        address: Address,
        attempts: usize,
        #[source]
        source: Box<TokenError>,
    },

    #[error("Event decoding error: {message}")]
    DecodeError { message: String },

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
            LocalDbError::MissingRunnerTarget {
                chain_id,
                orderbook_address,
            } => format!(
                "Missing runner target for chain {} orderbook {:#x}",
                chain_id, orderbook_address
            ),
            LocalDbError::RunnerNetworkChainIdMismatch {
                network_key,
                expected,
                found,
            } => format!(
                "Network '{}' has mismatched chain ids (expected {}, found {})",
                network_key, expected, found
            ),
            LocalDbError::DumpUrlConstructionFailed { url, source } => {
                format!("Failed to build dump url '{}': {}", url, source)
            }
            LocalDbError::DecodeError { message } => format!("Event decoding error: {}", message),
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

impl LocalDb {
    pub fn new_with_regular_rpc(url: Url) -> Result<Self, LocalDbError> {
        Self::new_with_regular_rpcs(vec![url])
    }

    pub fn new_with_regular_rpcs(urls: Vec<Url>) -> Result<Self, LocalDbError> {
        let rpc_client = RpcClient::new_with_urls(urls)?;
        Ok(Self { rpc_client })
    }

    pub fn new_with_hyper_rpc(chain_id: u32, api_token: String) -> Result<Self, LocalDbError> {
        let rpc_client = RpcClient::new_with_hyper_rpc(chain_id, &api_token)?;
        Ok(Self { rpc_client })
    }

    pub fn new(chain_id: u32, api_token: String) -> Result<Self, LocalDbError> {
        Self::new_with_hyper_rpc(chain_id, api_token)
    }

    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    pub fn check_support(chain_id: u32) -> bool {
        SUPPORTED_LOCAL_DB_CHAINS.contains(&chain_id)
    }

    #[cfg(test)]
    pub fn new_with_url(url: Url) -> Self {
        let rpc_client = RpcClient::new_with_urls(vec![url]).expect("create RPC client");
        Self { rpc_client }
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    pub fn update_rpc_urls(&mut self, urls: Vec<Url>) {
        self.rpc_client.update_rpc_urls(urls);
    }

    pub fn decode_events(
        &self,
        events: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
        decode_events_impl(events).map_err(|err| LocalDbError::DecodeError {
            message: err.to_string(),
        })
    }

    pub fn decoded_events_to_statements(
        &self,
        chain_id: u32,
        orderbook_address: Address,
        events: &[DecodedEventData<DecodedEvent>],
        decimals_by_token: &HashMap<Address, u8>,
    ) -> Result<SqlStatementBatch, LocalDbError> {
        decoded_events_to_statements_impl(chain_id, orderbook_address, events, decimals_by_token)
            .map_err(LocalDbError::InsertError)
    }

    pub fn raw_events_to_statements(
        &self,
        chain_id: u32,
        orderbook_address: Address,
        raw_events: &[LogEntryResponse],
    ) -> Result<SqlStatementBatch, LocalDbError> {
        raw_events_to_statements_impl(chain_id, orderbook_address, raw_events)
            .map_err(LocalDbError::InsertError)
    }
}

#[cfg(test)]
mod bool_deserialize_tests {
    use super::*;
    use alloy::primitives::{Address, Bytes, U256};
    use alloy::sol_types::SolEvent;
    use rain_orderbook_bindings::IOrderBookV5::{AddOrderV3, DepositV2};
    use std::collections::HashMap;
    use std::str::FromStr;

    fn make_local_db() -> LocalDb {
        LocalDb::new(8453, "test_token".to_string()).expect("create LocalDb")
    }

    fn sample_log_entry_with_invalid_data() -> LogEntryResponse {
        LogEntryResponse {
            address: "0x1111111111111111111111111111111111111111".to_string(),
            topics: vec![AddOrderV3::SIGNATURE_HASH.to_string()],
            data: "0xnothex".to_string(),
            block_number: "0x1".to_string(),
            block_timestamp: Some("0x2".to_string()),
            transaction_hash: "0x3".to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0x4".to_string(),
            log_index: "0x0".to_string(),
            removed: false,
        }
    }

    #[test]
    fn decode_events_maps_decode_errors() {
        let db = make_local_db();
        let event = sample_log_entry_with_invalid_data();

        let err = db.decode_events(&[event]).unwrap_err();
        match err {
            LocalDbError::DecodeError { message } => {
                assert!(
                    message.to_lowercase().contains("hex"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected LocalDbError::DecodeError, got {other:?}"),
        }
    }

    fn deposit_event_with_invalid_block() -> DecodedEventData<DecodedEvent> {
        let deposit = DepositV2 {
            sender: Address::from([0u8; 20]),
            token: Address::from([1u8; 20]),
            vaultId: U256::from(1u64).into(),
            depositAmountUint256: U256::from(10u64),
        };

        DecodedEventData {
            event_type: decode::EventType::DepositV2,
            block_number: "not-hex".to_string(),
            block_timestamp: "0x0".to_string(),
            transaction_hash: Bytes::from_str("0x50").unwrap(),
            log_index: "0x0".to_string(),
            decoded_data: DecodedEvent::DepositV2(Box::new(deposit)),
        }
    }

    #[test]
    fn decoded_events_to_statements_maps_insert_errors() {
        let db = make_local_db();
        let event = deposit_event_with_invalid_block();
        let mut decimals = HashMap::new();
        if let DecodedEvent::DepositV2(deposit) = &event.decoded_data {
            decimals.insert(deposit.token, 18);
        }

        let err = db
            .decoded_events_to_statements(1, Address::ZERO, &[event], &decimals)
            .unwrap_err();
        match err {
            LocalDbError::InsertError(..) => {}
            other => panic!("expected LocalDbError::InsertError, got {other:?}"),
        }
    }

    #[test]
    fn invalid_block_number_helper_preserves_source() {
        let source = ParseError::InvalidDigit('x');
        let err = LocalDbError::InvalidBlockNumber {
            value: "0xzz".to_string(),
            source,
        };

        match err {
            LocalDbError::InvalidBlockNumber {
                value,
                source: parse,
            } => {
                assert_eq!(value, "0xzz");
                assert_eq!(parse, source);
            }
            other => panic!("expected LocalDbError::InvalidBlockNumber, got {other:?}"),
        }
    }
}
