use alloy_ethers_typecast::transaction::ReadableClientError;
use alloy_primitives::hex::FromHexError;
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
use thiserror::Error;
use url::ParseError;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Error)]
pub enum FailedQuote {
    #[error("Order does not exist")]
    NonExistent,
    #[error(transparent)]
    RevertError(#[from] AbiDecodedErrorType),
    #[error("Corrupt return data: {0}")]
    CorruptReturnData(String),
    #[error(transparent)]
    RevertErrorDecodeFailed(#[from] AbiDecodeFailedErrors),
    #[cfg(target_family = "wasm")]
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    RpcCallError(#[from] ReadableClientError),
    #[error(transparent)]
    UrlParseError(#[from] ParseError),
    #[error(transparent)]
    SubgraphClientError(#[from] OrderbookSubgraphClientError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    AlloySolTypesError(#[from] alloy_sol_types::Error),
    #[cfg(target_family = "wasm")]
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}

#[cfg(target_family = "wasm")]
impl From<FailedQuote> for JsValue {
    fn from(value: FailedQuote) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

#[cfg(target_family = "wasm")]
impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
