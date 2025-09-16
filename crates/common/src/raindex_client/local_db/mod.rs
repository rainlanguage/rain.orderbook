pub mod decode;
pub mod fetch;
pub mod helpers;
pub mod insert;
pub mod query;
pub mod sync;
pub mod token_fetch;
pub mod tokens;

use super::*;
use crate::rpc_client::{RpcClient, RpcClientError};
use alloy::primitives::hex::FromHexError;
use alloy::primitives::ruint::ParseError;
pub use fetch::FetchConfig;
use query::LocalDbQueryError;

const SUPPORTED_LOCAL_DB_CHAINS: &[u32] = &[42161];

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LocalDb {
    rpc: RpcClient,
    rpc_urls: Vec<Url>,
}

impl Default for LocalDb {
    fn default() -> Self {
        Self {
            rpc: RpcClient,
            rpc_urls: vec![Url::parse("http://localhost:4444").unwrap()],
        }
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
    pub fn new_with_regular_rpc(url: Url) -> Self {
        Self {
            rpc: RpcClient,
            rpc_urls: vec![url],
        }
    }

    pub fn new_with_regular_rpcs(urls: Vec<Url>) -> Self {
        Self {
            rpc: RpcClient,
            rpc_urls: urls,
        }
    }

    pub fn new_with_hyper_rpc(chain_id: u32, api_token: String) -> Result<Self, LocalDbError> {
        let rpc_url = RpcClient::build_hyper_url(chain_id, &api_token)?;
        Ok(Self {
            rpc: RpcClient,
            rpc_urls: vec![rpc_url],
        })
    }

    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc
    }

    pub fn rpc_urls(&self) -> &[Url] {
        &self.rpc_urls
    }

    pub fn check_support(chain_id: u32) -> bool {
        SUPPORTED_LOCAL_DB_CHAINS.contains(&chain_id)
    }

    #[cfg(test)]
    pub fn new_with_url(url: Url) -> Self {
        Self {
            rpc: RpcClient,
            rpc_urls: vec![url],
        }
    }

    #[cfg(test)]
    pub fn set_rpc_url(&mut self, url: Url) {
        self.rpc_urls = vec![url];
    }
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "getLocalDbClient", preserve_js_class)]
    pub fn get_local_db_client(&self, chain_id: u32) -> Result<LocalDb, RaindexError> {
        let rpcs = self.get_rpc_urls_for_chain(chain_id)?;
        Ok(LocalDb::new_with_regular_rpcs(rpcs))
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
