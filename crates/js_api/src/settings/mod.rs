use std::str::FromStr;

use alloy::{hex::FromHexError, primitives::Address};
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg,
    orderbook::OrderbookCfg,
    yaml::{dotrain::DotrainYaml, orderbook::OrderbookYaml, YamlError, YamlParsable},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct Settings {
    settings: Vec<String>,
    selected_deployment: Option<String>,
}
impl PartialEq for Settings {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings && self.selected_deployment == other.selected_deployment
    }
}

impl Settings {
    fn get_dotrain_yaml(&self) -> Result<DotrainYaml, SettingsError> {
        Ok(DotrainYaml::new(self.settings.clone(), false)?)
    }

    fn get_orderbook_yaml(&self) -> Result<OrderbookYaml, SettingsError> {
        Ok(OrderbookYaml::new(self.settings.clone(), false)?)
    }

    fn get_deployment(&self) -> Result<DeploymentCfg, SettingsError> {
        let selected_deployment = self
            .selected_deployment
            .as_ref()
            .ok_or(SettingsError::NoDeploymentSelected)?;
        Ok(self
            .get_dotrain_yaml()?
            .get_deployment(selected_deployment)?)
    }
}

#[wasm_bindgen]
impl Settings {
    #[wasm_bindgen(constructor)]
    pub fn new(
        settings: Vec<String>,
        selected_deployment: Option<String>,
    ) -> Result<Self, SettingsError> {
        Ok(Self {
            settings,
            selected_deployment,
        })
    }

    #[wasm_bindgen(js_name = "getOrderbookByAddress")]
    pub fn get_orderbook_by_address(
        &self,
        orderbook_address: &str,
    ) -> Result<OrderbookCfg, SettingsError> {
        let address = Address::from_str(orderbook_address).map_err(SettingsError::FromHexError)?;
        let orderbook_yaml = self.get_orderbook_yaml()?;
        Ok(orderbook_yaml.get_orderbook_by_address(address)?)
    }

    #[wasm_bindgen(js_name = "getCurrentOrderbookForDeployment")]
    pub fn get_orderbook_for_current_deployment(&self) -> Result<OrderbookCfg, SettingsError> {
        let deployment = self.get_deployment()?;
        let orderbook = deployment
            .order
            .orderbook
            .clone()
            .ok_or(SettingsError::OrderbookNotFound(deployment.key))?;
        Ok(orderbook.as_ref().clone())
    }
}

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("No deployment selected")]
    NoDeploymentSelected,

    #[error("Orderbook not found in order for deployment {0}")]
    OrderbookNotFound(String),

    #[error("Orderbook yaml error: {0}")]
    YamlError(#[from] YamlError),
    #[error("Invalid address: {0}")]
    FromHexError(#[from] FromHexError),
}
impl From<SettingsError> for JsValue {
    fn from(value: SettingsError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
