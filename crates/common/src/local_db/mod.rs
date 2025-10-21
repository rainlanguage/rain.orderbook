pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;
pub mod sync;
pub mod token_fetch;
pub mod tokens;

use crate::rpc_client::{LogEntryResponse, RpcClient, RpcClientError};
use alloy::primitives::ruint::ParseError;
use alloy::primitives::{hex::FromHexError, Address};
use decode::{decode_events as decode_events_impl, DecodedEvent, DecodedEventData};
pub use fetch::FetchConfig;
use insert::{
    decoded_events_to_sql as decoded_events_to_sql_impl,
    raw_events_to_sql as raw_events_to_sql_impl,
};
use query::LocalDbQueryError;
use std::collections::HashMap;
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

    #[error("Event decoding error: {message}")]
    DecodeError { message: String },

    #[error("Database insertion error: {message}")]
    InsertError { message: String },

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
            LocalDbError::Config { message } => format!("Configuration error: {}", message),
            LocalDbError::DecodeError { message } => format!("Event decoding error: {}", message),
            LocalDbError::InsertError { message } => {
                format!("Database insertion error: {}", message)
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

    pub fn decoded_events_to_sql(
        &self,
        events: &[DecodedEventData<DecodedEvent>],
        end_block: u64,
        decimals_by_token: &HashMap<Address, u8>,
        prefix_sql: Option<&str>,
    ) -> Result<String, LocalDbError> {
        decoded_events_to_sql_impl(events, end_block, decimals_by_token, prefix_sql).map_err(
            |err| LocalDbError::InsertError {
                message: err.to_string(),
            },
        )
    }

    pub fn raw_events_to_sql(
        &self,
        raw_events: &[LogEntryResponse],
    ) -> Result<String, LocalDbError> {
        raw_events_to_sql_impl(raw_events).map_err(|err| LocalDbError::InsertError {
            message: err.to_string(),
        })
    }
}

#[cfg(test)]
mod bool_deserialize_tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use alloy::sol_types::SolEvent;
    use rain_orderbook_bindings::IOrderBookV5::{AddOrderV3, DepositV2};
    use std::collections::HashMap;

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
            transaction_hash: "0x5".to_string(),
            log_index: "0x0".to_string(),
            decoded_data: DecodedEvent::DepositV2(Box::new(deposit)),
        }
    }

    #[test]
    fn decoded_events_to_sql_maps_insert_errors() {
        let db = make_local_db();
        let event = deposit_event_with_invalid_block();
        let mut decimals = HashMap::new();
        if let DecodedEvent::DepositV2(deposit) = &event.decoded_data {
            decimals.insert(deposit.token, 18);
        }

        let err = db
            .decoded_events_to_sql(&[event], 42, &decimals, None)
            .unwrap_err();
        match err {
            LocalDbError::InsertError { message } => {
                assert!(
                    message.to_lowercase().contains("hex"),
                    "unexpected message: {}",
                    message
                );
            }
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
