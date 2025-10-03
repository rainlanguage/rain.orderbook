pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;
pub mod sync;

use super::*;
use crate::hyper_rpc::{HyperRpcClient, HyperRpcError, LogEntryResponse};
use alloy::primitives::hex::FromHexError;
use alloy::primitives::ruint::ParseError;
use decode::{decode_events as decode_events_impl, DecodedEvent, DecodedEventData};
pub use fetch::FetchConfig;
use insert::decoded_events_to_sql as decoded_events_to_sql_impl;
use query::LocalDbQueryError;
use url::Url;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LocalDb {
    client: HyperRpcClient,
}

#[derive(Debug, thiserror::Error)]
pub enum LocalDbError {
    #[error("{0}")]
    CustomError(String),

    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),

    #[error("RPC error")]
    Rpc(#[from] HyperRpcError),

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
    pub fn new(chain_id: u32, api_token: String) -> Result<Self, LocalDbError> {
        let client = HyperRpcClient::new(chain_id, api_token)?;
        Ok(Self { client })
    }

    pub fn new_with_additional_rpcs(
        chain_id: u32,
        api_token: String,
        additional_rpcs: Vec<Url>,
    ) -> Result<Self, LocalDbError> {
        let client =
            HyperRpcClient::new_with_additional_rpcs(chain_id, api_token, additional_rpcs)?;
        Ok(Self { client })
    }

    pub fn new_with_regular_rpc(url: Url) -> Result<Self, LocalDbError> {
        Self::new_with_regular_rpcs(vec![url])
    }

    pub fn new_with_regular_rpcs(urls: Vec<Url>) -> Result<Self, LocalDbError> {
        let client = HyperRpcClient::from_urls(0, urls)?;
        Ok(Self { client })
    }

    pub fn hyper_rpc_client(&self) -> &HyperRpcClient {
        &self.client
    }

    #[cfg(test)]
    pub fn new_with_client(client: HyperRpcClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub fn client_mut(&mut self) -> &mut HyperRpcClient {
        &mut self.client
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
    ) -> Result<String, LocalDbError> {
        decoded_events_to_sql_impl(events, end_block).map_err(|err| LocalDbError::InsertError {
            message: err.to_string(),
        })
    }
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "getLocalDbClient", preserve_js_class)]
    pub fn get_local_db_client(
        &self,
        chain_id: u32,
        api_token: String,
    ) -> Result<LocalDb, RaindexError> {
        LocalDb::new(chain_id, api_token).map_err(RaindexError::LocalDbError)
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
