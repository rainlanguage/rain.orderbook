use std::str::FromStr;

use super::*;
use alloy::primitives::{Address, U256};
use rain_orderbook_subgraph_client::types::common::SgTransaction;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct RaindexTransaction {
    id: String,
    from: Address,
    block_number: U256,
    timestamp: U256,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexTransaction {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn from(&self) -> String {
        self.from.to_string()
    }
    #[wasm_bindgen(getter = blockNumber)]
    pub fn block_number(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.block_number.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.timestamp.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexTransaction {
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn from(&self) -> Address {
        self.from
    }
    pub fn block_number(&self) -> U256 {
        self.block_number
    }
    pub fn timestamp(&self) -> U256 {
        self.timestamp
    }
}

impl TryFrom<SgTransaction> for RaindexTransaction {
    type Error = RaindexError;
    fn try_from(transaction: SgTransaction) -> Result<Self, Self::Error> {
        Ok(Self {
            id: transaction.id.0,
            from: Address::from_str(&transaction.from.0)?,
            block_number: U256::from_str(&transaction.block_number.0)?,
            timestamp: U256::from_str(&transaction.timestamp.0)?,
        })
    }
}
