use crate::transaction::TransactionArgsError;
use alloy_dyn_abi::JsonAbiExt;
use alloy_ethers_typecast::{client::LedgerClientError, transaction::WritableClientError};
use alloy_json_abi::Error as AlloyError;
use forker::ForkedEvm;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, PoisonError},
};
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
pub enum AbiDecodedErrorType {
    Unknown,
    Known {
        name: String,
        args: Vec<String>,
        sig: String,
    },
}

/// decodes an error returned from calling a contract by searching its selector in registry
pub async fn abi_decode_error<'a>(
    error_data: &[u8],
) -> Result<AbiDecodedErrorType, AbiDecodeFailedErrors<'a>> {
    let (hash_bytes, args_data) = error_data.split_at(4);
    let selector_hash = alloy_primitives::hex::encode_prefixed(hash_bytes);
    let selector_hash_bytes: [u8; 4] = hash_bytes.try_into()?;

    // check if selector already is cached
    {
        let selectors = SELECTORS.lock()?;
        if let Some(error) = selectors.get(&selector_hash_bytes) {
            if let Ok(result) = error.abi_decode_input(args_data, false) {
                return Ok(AbiDecodedErrorType::Known {
                    name: error.name.to_string(),
                    args: result.iter().map(|v| format!("{:?}", v)).collect(),
                    sig: error.signature(),
                });
            } else {
                return Ok(AbiDecodedErrorType::Unknown);
            }
        }
    };

    let client = Client::builder().build()?;
    let response = client
        .get(SELECTOR_REGISTRY_URL)
        .query(&vec![
            ("function", selector_hash.as_str()),
            ("filter", "true"),
        ])
        .header("accept", "application/json")
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(selectors) = response["result"]["function"][selector_hash].as_array() {
        for opt_selector in selectors {
            if let Some(selector) = opt_selector["name"].as_str() {
                if let Ok(error) = selector.parse::<AlloyError>() {
                    if let Ok(result) = error.abi_decode_input(args_data, false) {
                        // cache the fetched selector
                        {
                            let mut cached_selectors = SELECTORS.lock()?;
                            cached_selectors.insert(selector_hash_bytes, error.clone());
                        };
                        return Ok(AbiDecodedErrorType::Known {
                            sig: error.signature(),
                            name: error.name,
                            args: result.iter().map(|v| format!("{:?}", v)).collect(),
                        });
                    }
                }
            }
        }
        Ok(AbiDecodedErrorType::Unknown)
    } else {
        Ok(AbiDecodedErrorType::Unknown)
    }
}

#[derive(Debug)]
pub enum AbiDecodeFailedErrors<'a> {
    ReqwestError(reqwest::Error),
    InvalidSelectorHash(std::array::TryFromSliceError),
    SelectorsCachePoisoned(PoisonError<MutexGuard<'a, HashMap<[u8; 4], AlloyError>>>),
}

impl std::fmt::Display for AbiDecodeFailedErrors<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSelectorHash(v) => write!(f, "{}", v),
            Self::SelectorsCachePoisoned(v) => write!(f, "{}", v),
            Self::ReqwestError(v) => write!(f, "{}", v),
        }
    }
}

impl std::error::Error for AbiDecodeFailedErrors<'_> {}

impl From<std::array::TryFromSliceError> for AbiDecodeFailedErrors<'_> {
    fn from(value: std::array::TryFromSliceError) -> Self {
        Self::InvalidSelectorHash(value)
    }
}

impl From<reqwest::Error> for AbiDecodeFailedErrors<'_> {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, HashMap<[u8; 4], AlloyError>>>>
    for AbiDecodeFailedErrors<'a>
{
    fn from(value: PoisonError<MutexGuard<'a, HashMap<[u8; 4], AlloyError>>>) -> Self {
        Self::SelectorsCachePoisoned(value)
    }
}

#[derive(Debug)]
pub enum ForkCallError<'a> {
    EVMError(String),
    AbiDecodeFailed(AbiDecodeFailedErrors<'a>),
    ForkCachePoisoned(PoisonError<MutexGuard<'a, HashMap<String, ForkedEvm>>>),
}

impl std::fmt::Display for ForkCallError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbiDecodeFailed(v) => write!(f, "{}", v),
            Self::ForkCachePoisoned(v) => write!(f, "{}", v),
            Self::EVMError(v) => write!(f, "{}", v),
        }
    }
}

impl std::error::Error for ForkCallError<'_> {}

impl<'a> From<AbiDecodeFailedErrors<'a>> for ForkCallError<'a> {
    fn from(value: AbiDecodeFailedErrors<'a>) -> Self {
        Self::AbiDecodeFailed(value)
    }
}

impl From<String> for ForkCallError<'_> {
    fn from(value: String) -> Self {
        Self::EVMError(value)
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, HashMap<String, ForkedEvm>>>> for ForkCallError<'a> {
    fn from(value: PoisonError<MutexGuard<'a, HashMap<String, ForkedEvm>>>) -> Self {
        Self::ForkCachePoisoned(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_decoder() {
        let res = abi_decode_error(&[26, 198, 105, 8])
            .await
            .expect("failed to get error selector");
        assert_eq!(
            AbiDecodedErrorType::Known {
                name: "UnexpectedOperandValue".to_owned(),
                args: vec![],
                sig: "UnexpectedOperandValue()".to_owned(),
            },
            res
        );
    }
}
