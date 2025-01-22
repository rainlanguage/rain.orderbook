use alloy::{hex::FromHexError, primitives::ruint::ParseError};
use rain_orderbook_common::{
    dotrain_order::calldata::DotrainOrderCalldataError,
    transaction::WritableTransactionExecuteError,
};
use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
use thiserror::Error;
use wasm_bindgen::{JsError, JsValue};

pub mod order;
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
    DotrainOrderCalldataError(#[from] DotrainOrderCalldataError),
    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}
impl From<SubgraphError> for JsValue {
    fn from(value: SubgraphError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
