pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;

use crate::hyper_rpc::{HyperRpcClient, HyperRpcError};
use alloy::primitives::ruint::ParseError;
pub use fetch::FetchConfig;

#[derive(Debug, Clone)]
pub struct SqliteWeb {
    client: HyperRpcClient,
}

#[derive(Debug, thiserror::Error)]
pub enum SqliteWebError {
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
}

impl SqliteWebError {
    pub fn invalid_block_number(value: impl Into<String>, source: ParseError) -> Self {
        SqliteWebError::InvalidBlockNumber {
            value: value.into(),
            source,
        }
    }
}

impl SqliteWeb {
    pub fn new(chain_id: u32, api_token: String) -> Result<Self, SqliteWebError> {
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
