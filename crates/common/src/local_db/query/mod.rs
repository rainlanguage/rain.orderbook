pub mod clear_tables;
pub mod create_tables;
pub mod fetch_erc20_tokens_by_addresses;
pub mod fetch_last_synced_block;
pub mod fetch_order_trades;
pub mod fetch_order_trades_count;
pub mod fetch_orders;
pub mod fetch_store_addresses;
pub mod fetch_tables;
pub mod fetch_vault;
pub mod fetch_vault_balance_changes;
pub mod fetch_vaults;
pub mod update_last_synced_block;

use serde::de::DeserializeOwned;
use thiserror::Error;

/// Backend-neutral error representing failures when executing SQL against the
/// embedded local database. Specific backends should map their native error
/// types into this structure.
#[derive(Debug, Error, Clone)]
pub enum LocalDbQueryError {
    #[error("Database operation failed: {message}")]
    Database { message: String },

    #[error("Invalid response format from database")]
    InvalidResponse,

    #[error("Deserialization failed: {message}")]
    Deserialization { message: String },
}

impl LocalDbQueryError {
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn invalid_response() -> Self {
        Self::InvalidResponse
    }

    pub fn deserialization(message: impl Into<String>) -> Self {
        Self::Deserialization {
            message: message.into(),
        }
    }
}

/// Helper trait for types that can be deserialized from DB JSON emitted by the
/// local database backend. Implementors must be deserializable from JSON.
pub trait FromDbJson: DeserializeOwned {}

impl<T> FromDbJson for T where T: DeserializeOwned {}
