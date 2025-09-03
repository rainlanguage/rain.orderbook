pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;
pub mod sync;

use super::*;
use crate::rpc_client::{RpcClient, RpcClientError};
use alloy::primitives::hex::FromHexError;
use alloy::primitives::ruint::ParseError;
pub use fetch::FetchConfig;
use query::LocalDbQueryError;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LocalDb {
    rpc: RpcClient,
    rpc_url: Url,
}

impl Default for LocalDb {
    fn default() -> Self {
        Self {
            rpc: RpcClient,
            rpc_url: Url::parse("http://localhost:4444").unwrap(),
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
            rpc_url: url,
        }
    }

    pub fn new_with_hyper_rpc(chain_id: u32, api_token: String) -> Result<Self, LocalDbError> {
        let rpc_url = RpcClient::build_hyper_url(chain_id, &api_token)?;
        Ok(Self {
            rpc: RpcClient,
            rpc_url,
        })
    }

    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc
    }

    pub fn rpc_url(&self) -> &Url {
        &self.rpc_url
    }

    #[cfg(test)]
    pub fn new_with_url(url: Url) -> Self {
        Self {
            rpc: RpcClient,
            rpc_url: url,
        }
    }

    #[cfg(test)]
    pub fn set_rpc_url(&mut self, url: Url) {
        self.rpc_url = url;
    }
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "getLocalDbClient", preserve_js_class)]
    pub fn get_local_db_client(&self, chain_id: u32) -> Result<LocalDb, RaindexError> {
        let rpcs = self.get_rpc_urls_for_chain(chain_id)?;
        // TODO: support multiple RPC URLs
        Ok(LocalDb::new_with_regular_rpc(rpcs[0].clone()))
    }
}
