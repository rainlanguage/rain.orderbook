use rain_orderbook_common::erc20::ERC20;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

mod erc20;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Account not found")]
    AccountNotFound,
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    #[error("Could not get account balance")]
    AccountBalanceError,
    #[error("Network connection error: {0}")]
    NetworkError(String),
    #[error("JavaScript error: {0}")]
    JsError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    FromHexError(#[from] alloy::hex::FromHexError),
    #[error(transparent)]
    ReadableClientError(#[from] alloy_ethers_typecast::transaction::ReadableClientError),
    #[error(transparent)]
    ERC20Error(#[from] rain_orderbook_common::erc20::Error),
    #[error(transparent)]
    SolTypesError(#[from] alloy::sol_types::Error),
    #[error(transparent)]
    ParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}

impl From<AccountError> for JsValue {
    fn from(value: AccountError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<AccountError> for WasmEncodedError {
    fn from(value: AccountError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_string(),
        }
    }
}
