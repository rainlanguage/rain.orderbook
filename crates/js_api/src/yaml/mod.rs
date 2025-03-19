use std::str::FromStr;

use alloy::{hex::FromHexError, primitives::Address};
use rain_orderbook_app_settings::{
    orderbook::OrderbookCfg,
    subgraph::SubgraphCfg,
    yaml::{orderbook::OrderbookYaml as OrderbookYamlCfg, YamlError, YamlParsable},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct OrderbookYaml {
    yaml: Vec<String>,
}
impl PartialEq for OrderbookYaml {
    fn eq(&self, other: &Self) -> bool {
        self.yaml == other.yaml
    }
}

impl OrderbookYaml {
    fn get_orderbook_yaml_cfg(&self) -> Result<OrderbookYamlCfg, OrderbookYamlError> {
        Ok(OrderbookYamlCfg::new(self.yaml.clone(), false)?)
    }
}

#[wasm_bindgen]
impl OrderbookYaml {
    #[wasm_bindgen(constructor)]
    pub fn new(yaml: Vec<String>) -> Result<Self, OrderbookYamlError> {
        Ok(Self { yaml })
    }

    #[wasm_bindgen(js_name = "getOrderbookByAddress")]
    pub fn get_orderbook_by_address(
        &self,
        orderbook_address: &str,
    ) -> Result<OrderbookCfg, OrderbookYamlError> {
        let address =
            Address::from_str(orderbook_address).map_err(OrderbookYamlError::FromHexError)?;
        let orderbook_yaml = self.get_orderbook_yaml_cfg()?;
        Ok(orderbook_yaml.get_orderbook_by_address(address)?)
    }
}

#[derive(Error, Debug)]
pub enum OrderbookYamlError {
    #[error("No deployment selected")]
    OrderbookNotFound(String),

    #[error("Orderbook yaml error: {0}")]
    YamlError(#[from] YamlError),
    #[error("Invalid address: {0}")]
    FromHexError(#[from] FromHexError),
}
impl From<OrderbookYamlError> for JsValue {
    fn from(value: OrderbookYamlError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
