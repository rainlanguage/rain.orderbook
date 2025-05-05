use alloy::{hex::FromHexError, primitives::ruint::ParseError};
use rain_orderbook_common::{deposit::DepositError, transaction::WritableTransactionExecuteError};
use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

pub mod add_order;
pub mod order;
pub mod remove_order;
pub mod transaction;
pub mod vault;

#[derive(Error, Debug)]
pub enum SubgraphError {
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Invalid output index")]
    InvalidOutputIndex,
    #[error("Invalid input index")]
    InvalidInputIndex,
    #[error(transparent)]
    OrderbookSubgraphClientError(#[from] OrderbookSubgraphClientError),
    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
    #[error(transparent)]
    DepositError(#[from] DepositError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}
impl From<SubgraphError> for JsValue {
    fn from(value: SubgraphError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
impl From<SubgraphError> for WasmEncodedError {
    fn from(value: SubgraphError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_string(),
        }
    }
}
