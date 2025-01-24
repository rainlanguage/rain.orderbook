use alloy::primitives::Address;
use alloy_ethers_typecast::transaction::ReadableClientError;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rain_orderbook_app_settings::{
    deployment::Deployment,
    gui::{
        Gui, GuiDeployment, GuiFieldDefinition, GuiPreset, NameAndDescription,
        ParseGuiConfigSourceError,
    },
    network::Network,
    order::Order,
    yaml::YamlError,
};
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};
use rain_orderbook_common::{
    dotrain_order::{calldata::DotrainOrderCalldataError, DotrainOrder, DotrainOrderError},
    erc20::ERC20,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::prelude::*;
use thiserror::Error;

mod deposits;
mod field_values;
mod order_operations;
mod select_tokens;
mod state_management;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct DeploymentKeys(Vec<String>);
impl_all_wasm_traits!(DeploymentKeys);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TokenInfo {
    pub address: Address,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
impl_all_wasm_traits!(TokenInfo);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct DeploymentDetails(BTreeMap<String, NameAndDescription>);
impl_all_wasm_traits!(DeploymentDetails);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct DotrainOrderGui {
    dotrain_order: DotrainOrder,
    selected_deployment: String,
    field_values: BTreeMap<String, field_values::PairValue>,
    deposits: BTreeMap<String, field_values::PairValue>,
}
#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "getDeploymentKeys")]
    pub async fn get_deployment_keys(dotrain: String) -> Result<DeploymentKeys, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        let keys = Gui::parse_deployment_keys(dotrain_order.dotrain_yaml().documents.clone())?;
        Ok(DeploymentKeys(keys))
    }

    #[wasm_bindgen(js_name = "chooseDeployment")]
    pub async fn choose_deployment(
        dotrain: String,
        deployment_name: String,
    ) -> Result<DotrainOrderGui, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;

        let keys = Gui::parse_deployment_keys(dotrain_order.dotrain_yaml().documents.clone())?;
        if !keys.contains(&deployment_name) {
            return Err(GuiError::DeploymentNotFound(deployment_name.clone()));
        }

        Ok(Self {
            dotrain_order,
            selected_deployment: deployment_name.clone(),
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
        })
    }

    #[wasm_bindgen(js_name = "getGuiConfig")]
    pub fn get_gui_config(&self) -> Result<Gui, GuiError> {
        let gui = self
            .dotrain_order
            .dotrain_yaml()
            .get_gui(Some(self.selected_deployment.clone()))?
            .ok_or(GuiError::GuiConfigNotFound)?;
        Ok(gui)
    }

    #[wasm_bindgen(js_name = "getCurrentDeployment")]
    pub fn get_current_deployment(&self) -> Result<GuiDeployment, GuiError> {
        let gui = self.get_gui_config()?;
        let (_, gui_deployment) = gui
            .deployments
            .into_iter()
            .find(|(name, _)| name == &self.selected_deployment)
            .ok_or(GuiError::DeploymentNotFound(
                self.selected_deployment.clone(),
            ))?;
        Ok(gui_deployment.clone())
    }

    /// Get token info for a given key
    ///
    /// Returns a [`TokenInfo`]
    #[wasm_bindgen(js_name = "getTokenInfo")]
    pub async fn get_token_info(&self, key: String) -> Result<TokenInfo, GuiError> {
        let token = self.dotrain_order.orderbook_yaml().get_token(&key)?;

        let token_info = if token.decimals.is_some()
            && token.label.is_some()
            && token.symbol.is_some()
        {
            TokenInfo {
                address: token.address,
                decimals: token.decimals.unwrap(),
                name: token.label.unwrap(),
                symbol: token.symbol.unwrap(),
            }
        } else {
            let order_key = Deployment::parse_order_key(
                self.dotrain_order.dotrain_yaml().documents,
                &self.selected_deployment,
            )?;
            let network_key =
                Order::parse_network_key(self.dotrain_order.dotrain_yaml().documents, &order_key)?;
            let rpc_url =
                Network::parse_rpc(self.dotrain_order.dotrain_yaml().documents, &network_key)?;

            let erc20 = ERC20::new(rpc_url, token.address);
            let onchain_info = erc20.token_info(None).await?;

            TokenInfo {
                address: token.address,
                decimals: token.decimals.unwrap_or(onchain_info.decimals),
                name: token.label.unwrap_or(onchain_info.name),
                symbol: token.symbol.unwrap_or(onchain_info.symbol),
            }
        };

        Ok(token_info)
    }

    #[wasm_bindgen(js_name = "getStrategyDetails")]
    pub async fn get_strategy_details(dotrain: String) -> Result<NameAndDescription, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        let details = Gui::parse_strategy_details(dotrain_order.dotrain_yaml().documents.clone())?;
        Ok(details)
    }

    #[wasm_bindgen(js_name = "getDeploymentDetails")]
    pub async fn get_deployment_details(dotrain: String) -> Result<DeploymentDetails, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        let deployment_details =
            Gui::parse_deployment_details(dotrain_order.dotrain_yaml().documents.clone())?;
        Ok(DeploymentDetails(deployment_details.into_iter().collect()))
    }
}

#[derive(Error, Debug)]
pub enum GuiError {
    #[error("Gui config not found")]
    GuiConfigNotFound,
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),
    #[error("Field binding not found: {0}")]
    FieldBindingNotFound(String),
    #[error("Deposit token not found in gui config: {0}")]
    DepositTokenNotFound(String),
    #[error("Orderbook not found")]
    OrderbookNotFound,
    #[error("Deserialized dotrain mismatch")]
    DotrainMismatch,
    #[error("Vault id not found for output index: {0}")]
    VaultIdNotFound(String),
    #[error("Deployer not found")]
    DeployerNotFound,
    #[error("Token not found {0}")]
    TokenNotFound(String),
    #[error("Invalid preset")]
    InvalidPreset,
    #[error("Select tokens not set")]
    SelectTokensNotSet,
    #[error("Token must be selected: {0}")]
    TokenMustBeSelected(String),
    #[error("Binding has no presets: {0}")]
    BindingHasNoPresets(String),
    #[error("Token not in select tokens: {0}")]
    TokenNotInSelectTokens(String),
    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),
    #[error(transparent)]
    ParseGuiConfigSourceError(#[from] ParseGuiConfigSourceError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    FromHexError(#[from] alloy::hex::FromHexError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    DepositError(#[from] rain_orderbook_common::deposit::DepositError),
    #[error(transparent)]
    ParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error(transparent)]
    ReadContractParametersBuilderError(
        #[from] alloy_ethers_typecast::transaction::ReadContractParametersBuilderError,
    ),
    #[error(transparent)]
    UnitsError(#[from] alloy::primitives::utils::UnitsError),
    #[error(transparent)]
    WritableTransactionExecuteError(
        #[from] rain_orderbook_common::transaction::WritableTransactionExecuteError,
    ),
    #[error(transparent)]
    AddOrderArgsError(#[from] rain_orderbook_common::add_order::AddOrderArgsError),
    #[error(transparent)]
    ERC20Error(#[from] rain_orderbook_common::erc20::Error),
    #[error(transparent)]
    SolTypesError(#[from] alloy::sol_types::Error),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
    #[error(transparent)]
    DotrainOrderCalldataError(#[from] DotrainOrderCalldataError),
    #[error(transparent)]
    YamlError(#[from] YamlError),
}
impl From<GuiError> for JsValue {
    fn from(value: GuiError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
