use std::{collections::HashMap, sync::Mutex};

use crate::transaction::TransactionArgsError;
use alloy_dyn_abi::JsonAbiExt;
use alloy_ethers_typecast::{client::LedgerClientError, transaction::WritableClientError};
use alloy_json_abi::Error as AlloyError;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

pub const SELECTOR_REGISTRY_URL: &str = "https://api.openchain.xyz/signature-database/v1/lookup";

/// hashmap of cached error selectors    
pub static SELECTORS: Lazy<Mutex<HashMap<[u8; 4], AlloyError>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Error, Debug)]
pub enum WritableTransactionExecuteError {
    #[error("WritableClient error: {0}")]
    WritableClient(#[from] WritableClientError),
    #[error("TransactionArgs error: {0}")]
    TransactionArgs(#[from] TransactionArgsError),
    #[error("LedgerClient error: {0}")]
    LedgerClient(#[from] LedgerClientError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DecodedErrorType {
    Unknown,
    Known {
        name: String,
        args: Vec<String>,
        sig: String,
    },
}

/// decodes an error returned from calling a contract by searching its selector in registry
pub async fn decode_error(error_data: &[u8]) -> Result<DecodedErrorType, String> {
    let (hash_bytes, args_data) = error_data.split_at(4);
    let selector_hash = alloy_primitives::hex::encode_prefixed(hash_bytes);
    let selector_hash_bytes: [u8; 4] = hash_bytes
        .try_into()
        .or(Err("provided data contains no selector".to_owned()))?;

    // check if selector already is cached
    {
        let selectors = SELECTORS.lock().unwrap();
        if let Some(error) = selectors.get(&selector_hash_bytes) {
            if let Ok(result) = error.abi_decode_input(args_data, false) {
                return Ok(DecodedErrorType::Known {
                    name: error.name.to_string(),
                    args: result.iter().map(|v| format!("{:?}", v)).collect(),
                    sig: error.signature(),
                });
            } else {
                return Ok(DecodedErrorType::Unknown);
            }
        }
    };

    let client = Client::builder().build().unwrap();
    let response = client
        .get(SELECTOR_REGISTRY_URL)
        .query(&vec![
            ("function", selector_hash.as_str()),
            ("filter", "true"),
        ])
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Value>()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(selectors) = response["result"]["function"][selector_hash].as_array() {
        for opt_selector in selectors {
            if let Some(selector) = opt_selector["name"].as_str() {
                if let Ok(error) = selector.parse::<AlloyError>() {
                    if let Ok(result) = error.abi_decode_input(args_data, false) {
                        // cache the fetched selector
                        {
                            let mut cached_selectors = SELECTORS.lock().unwrap();
                            cached_selectors.insert(selector_hash_bytes, error.clone());
                        };
                        return Ok(DecodedErrorType::Known {
                            sig: error.signature(),
                            name: error.name,
                            args: result.iter().map(|v| format!("{:?}", v)).collect(),
                        });
                    }
                }
            }
        }
        Ok(DecodedErrorType::Unknown)
    } else {
        Ok(DecodedErrorType::Unknown)
    }
}
