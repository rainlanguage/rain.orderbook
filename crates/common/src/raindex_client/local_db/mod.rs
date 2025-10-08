pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;
pub mod sync;
pub mod token_fetch;
pub mod tokens;

use super::*;
use crate::rpc_client::{LogEntryResponse, RpcClient, RpcClientError};
use alloy::primitives::ruint::ParseError;
use alloy::primitives::{hex::FromHexError, Address};
use decode::{decode_events as decode_events_impl, DecodedEvent, DecodedEventData};
pub use fetch::FetchConfig;
use insert::decoded_events_to_sql as decoded_events_to_sql_impl;
use query::LocalDbQueryError;
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LocalDb {
    rpc_client: RpcClient,
}

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

    #[error("Failed to load orderbook configuration")]
    OrderbookConfigNotFound(#[source] Box<RaindexError>),

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
            LocalDbError::OrderbookConfigNotFound(err) => {
                format!("Failed to load orderbook configuration: {}", err)
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

impl From<LocalDbError> for WasmEncodedError {
    fn from(value: LocalDbError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

impl LocalDbError {
    pub fn invalid_block_number(value: impl Into<String>, source: ParseError) -> Self {
        LocalDbError::InvalidBlockNumber {
            value: value.into(),
            source,
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
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "getLocalDbClient", preserve_js_class)]
    pub fn get_local_db_client(&self, chain_id: u32) -> Result<LocalDb, RaindexError> {
        let rpcs = self.get_rpc_urls_for_chain(chain_id)?;
        LocalDb::new_with_regular_rpcs(rpcs).map_err(RaindexError::LocalDbError)
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
        let err = LocalDbError::invalid_block_number("0xzz", source);

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

// Shared serde helper: accept 0/1 integers, booleans, or strings for booleans.
// Useful for SQLite queries that emit 0/1 rather than true/false.
pub fn bool_from_int_or_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Unexpected};
    struct BoolOrIntVisitor;

    impl<'de> serde::de::Visitor<'de> for BoolOrIntVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or 0/1 integer")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::invalid_value(
                    Unexpected::Unsigned(v),
                    &"0 or 1 for boolean",
                )),
            }
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::invalid_value(
                    Unexpected::Signed(v),
                    &"0 or 1 for boolean",
                )),
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v.to_ascii_lowercase().as_str() {
                "true" | "1" => Ok(true),
                "false" | "0" => Ok(false),
                _ => Err(E::invalid_value(
                    Unexpected::Str(v),
                    &"'true'/'false' or '1'/'0'",
                )),
            }
        }
    }

    deserializer.deserialize_any(BoolOrIntVisitor)
}

#[cfg(test)]
mod tests {
    use super::bool_from_int_or_bool;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Wrap {
        #[serde(deserialize_with = "bool_from_int_or_bool")]
        b: bool,
    }

    #[test]
    fn deserializes_bool_values() {
        let t: Wrap = serde_json::from_str("{\"b\": true}").unwrap();
        assert!(t.b);
        let f: Wrap = serde_json::from_str("{\"b\": false}").unwrap();
        assert!(!f.b);
    }

    #[test]
    fn deserializes_int_values() {
        let t: Wrap = serde_json::from_str("{\"b\": 1}").unwrap();
        assert!(t.b);
        let f: Wrap = serde_json::from_str("{\"b\": 0}").unwrap();
        assert!(!f.b);
    }

    #[test]
    fn deserializes_string_values() {
        let t1: Wrap = serde_json::from_str("{\"b\": \"true\"}").unwrap();
        assert!(t1.b);
        let t2: Wrap = serde_json::from_str("{\"b\": \"1\"}").unwrap();
        assert!(t2.b);
        let f1: Wrap = serde_json::from_str("{\"b\": \"false\"}").unwrap();
        assert!(!f1.b);
        let f2: Wrap = serde_json::from_str("{\"b\": \"0\"}").unwrap();
        assert!(!f2.b);
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(serde_json::from_str::<Wrap>("{\"b\": 2}").is_err());
        assert!(serde_json::from_str::<Wrap>("{\"b\": -1}").is_err());
        assert!(serde_json::from_str::<Wrap>("{\"b\": \"yes\"}").is_err());
        assert!(serde_json::from_str::<Wrap>("{\"b\": \"maybe\"}").is_err());
    }
}
