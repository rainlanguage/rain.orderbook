pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;
pub mod sync;

use super::*;
use crate::hyper_rpc::{HyperRpcClient, HyperRpcError};
use alloy::primitives::hex::FromHexError;
use alloy::primitives::ruint::ParseError;
pub use fetch::FetchConfig;
use query::LocalDbQueryError;

#[derive(Debug, Clone, Default)]
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
