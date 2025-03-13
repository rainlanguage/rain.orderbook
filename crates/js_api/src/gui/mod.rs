use alloy::primitives::Address;
use alloy_ethers_typecast::transaction::ReadableClientError;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg,
    gui::{
        GuiCfg, GuiDeploymentCfg, GuiFieldDefinitionCfg, GuiPresetCfg, NameAndDescriptionCfg,
        ParseGuiConfigSourceError,
    },
    network::NetworkCfg,
    order::OrderCfg,
    yaml::{dotrain::DotrainYaml, YamlError, YamlParsable},
};
use rain_orderbook_common::{
    dotrain::{types::patterns::FRONTMATTER_SEPARATOR, RainDocument},
    dotrain_order::{DotrainOrder, DotrainOrderError},
    erc20::ERC20,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::prelude::*;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

mod deposits;
mod field_values;
mod order_operations;
mod select_tokens;
mod state_management;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TokenInfo {
    #[tsify(type = "string")]
    pub address: Address,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct DotrainOrderGui {
    dotrain_order: DotrainOrder,
    selected_deployment: String,
    field_values: BTreeMap<String, field_values::PairValue>,
    deposits: BTreeMap<String, field_values::PairValue>,
    #[serde(skip)]
    state_update_callback: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DotrainOrderGui {
        Self {
            dotrain_order: DotrainOrder::dummy(),
            selected_deployment: "".to_string(),
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            state_update_callback: None,
        }
    }
}

#[wasm_export]
impl DotrainOrderGui {
    #[wasm_export(js_name = "getDeploymentKeys", unchecked_return_type = "string[]")]
    pub async fn get_deployment_keys(dotrain: String) -> Result<Vec<String>, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        Ok(GuiCfg::parse_deployment_keys(
            dotrain_order.dotrain_yaml().documents.clone(),
        )?)
    }

    #[wasm_export(js_name = "chooseDeployment", unchecked_return_type = "void")]
    pub async fn choose_deployment(
        &mut self,
        dotrain: String,
        deployment_name: String,
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<(), GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;

        let keys = GuiCfg::parse_deployment_keys(dotrain_order.dotrain_yaml().documents.clone())?;
        if !keys.contains(&deployment_name) {
            return Err(GuiError::DeploymentNotFound(deployment_name.clone()));
        }

        self.dotrain_order = dotrain_order;
        self.selected_deployment = deployment_name;
        self.state_update_callback = state_update_callback;

        Ok(())
    }

    #[wasm_export(js_name = "getGuiConfig", unchecked_return_type = "GuiCfg")]
    pub fn get_gui_config(&self) -> Result<GuiCfg, GuiError> {
        let gui = self
            .dotrain_order
            .dotrain_yaml()
            .get_gui(Some(self.selected_deployment.clone()))?
            .ok_or(GuiError::GuiConfigNotFound)?;
        Ok(gui)
    }

    #[wasm_export(
        js_name = "getCurrentDeployment",
        unchecked_return_type = "GuiDeploymentCfg"
    )]
    pub fn get_current_deployment(&self) -> Result<GuiDeploymentCfg, GuiError> {
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
    #[wasm_export(js_name = "getTokenInfo", unchecked_return_type = "TokenInfo")]
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
            let order_key = DeploymentCfg::parse_order_key(
                self.dotrain_order.dotrain_yaml().documents,
                &self.selected_deployment,
            )?;
            let network_key = OrderCfg::parse_network_key(
                self.dotrain_order.dotrain_yaml().documents,
                &order_key,
            )?;
            let rpc_url =
                NetworkCfg::parse_rpc(self.dotrain_order.dotrain_yaml().documents, &network_key)?;

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

    #[wasm_export(js_name = "getAllTokenInfos", unchecked_return_type = "TokenInfo[]")]
    pub async fn get_all_token_infos(&self) -> Result<Vec<TokenInfo>, GuiError> {
        let select_tokens = self.get_select_tokens()?;

        let token_keys = match select_tokens.is_empty() {
            true => {
                let order_key = DeploymentCfg::parse_order_key(
                    self.dotrain_order.dotrain_yaml().documents,
                    &self.selected_deployment,
                )?;
                OrderCfg::parse_io_token_keys(
                    self.dotrain_order.dotrain_yaml().documents,
                    &order_key,
                )?
            }
            false => select_tokens
                .iter()
                .map(|token| token.key.clone())
                .collect(),
        };

        let mut result = Vec::new();
        for key in token_keys.iter() {
            result.push(self.get_token_info(key.clone()).await?);
        }
        Ok(result)
    }

    #[wasm_export(
        js_name = "getStrategyDetails",
        unchecked_return_type = "NameAndDescriptionCfg"
    )]
    pub async fn get_strategy_details(dotrain: String) -> Result<NameAndDescriptionCfg, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        let details =
            GuiCfg::parse_strategy_details(dotrain_order.dotrain_yaml().documents.clone())?;
        Ok(details)
    }

    #[wasm_export(
        js_name = "getDeploymentDetails",
        unchecked_return_type = "Map<string, NameAndDescriptionCfg>"
    )]
    pub async fn get_deployment_details(
        dotrain: String,
    ) -> Result<HashMap<String, NameAndDescriptionCfg>, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        Ok(GuiCfg::parse_deployment_details(
            dotrain_order.dotrain_yaml().documents.clone(),
        )?)
    }

    #[wasm_export(
        js_name = "getDeploymentDetail",
        unchecked_return_type = "NameAndDescriptionCfg"
    )]
    pub async fn get_deployment_detail(
        dotrain: String,
        key: String,
    ) -> Result<NameAndDescriptionCfg, GuiError> {
        let deployment_details = DotrainOrderGui::get_deployment_details(dotrain).await?;
        let deployment_detail = deployment_details
            .get(&key)
            .ok_or(GuiError::DeploymentNotFound(key))?;
        Ok(deployment_detail.clone())
    }

    #[wasm_export(js_name = "generateDotrainText", unchecked_return_type = "string")]
    pub fn generate_dotrain_text(&self) -> Result<String, GuiError> {
        let rain_document = RainDocument::create(self.dotrain_order.dotrain(), None, None, None);
        let dotrain = format!(
            "{}\n{}\n{}",
            DotrainYaml::get_yaml_string(self.dotrain_order.dotrain_yaml().documents[0].clone(),)?,
            FRONTMATTER_SEPARATOR,
            rain_document.body()
        );
        Ok(dotrain)
    }

    #[wasm_export(js_name = "getComposedRainlang", unchecked_return_type = "string")]
    pub async fn get_composed_rainlang(&mut self) -> Result<String, GuiError> {
        self.update_scenario_bindings()?;
        let dotrain = self.generate_dotrain_text()?;
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        let rainlang = dotrain_order
            .compose_deployment_to_rainlang(self.selected_deployment.clone())
            .await?;
        Ok(rainlang)
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
    #[error("Missing field value: {0}")]
    FieldValueNotSet(String),
    #[error("Deposit token not found in gui config: {0}")]
    DepositTokenNotFound(String),
    #[error("Missing deposit with token: {0}")]
    DepositNotSet(String),
    #[error("Orderbook not found")]
    OrderbookNotFound,
    #[error("Order not found: {0}")]
    OrderNotFound(String),
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
    #[error("Presets not set")]
    PresetsNotSet,
    #[error("Select tokens not set")]
    SelectTokensNotSet,
    #[error("Token must be selected: {0}")]
    TokenMustBeSelected(String),
    #[error("Binding has no presets: {0}")]
    BindingHasNoPresets(String),
    #[error("Token not in select tokens: {0}")]
    TokenNotInSelectTokens(String),
    #[error("JavaScript error: {0}")]
    JsError(String),
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
    YamlError(#[from] YamlError),
}

impl GuiError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            GuiError::GuiConfigNotFound => 
                "The GUI configuration could not be found. Please check your YAML configuration file.".to_string(),
            GuiError::DeploymentNotFound(name) => 
                format!("The deployment '{}' could not be found. Please select a valid deployment from your YAML configuration.", name),
            GuiError::FieldBindingNotFound(field) => 
                format!("The field binding '{}' could not be found in the YAML configuration.", field),
            GuiError::FieldValueNotSet(field) => 
                format!("The value for field '{}' is required but has not been set.", field),
            GuiError::DepositTokenNotFound(token) => 
                format!("The deposit token '{}' was not found in the YAML configuration.", token),
            GuiError::DepositNotSet(token) => 
                format!("A deposit for token '{}' is required but has not been set.", token),
            GuiError::OrderbookNotFound => 
                "The orderbook configuration could not be found. Please check your YAML configuration.".to_string(),
            GuiError::OrderNotFound(order) => 
                format!("The order '{}' could not be found in the YAML configuration.", order),
            GuiError::DotrainMismatch => 
                "There was a mismatch in the dotrain configuration. Please check your YAML configuration for consistency.".to_string(),
            GuiError::VaultIdNotFound(index) => 
                format!("The vault ID for output index '{}' could not be found in the YAML configuration.", index),
            GuiError::DeployerNotFound => 
                "The deployer configuration could not be found. Please check your YAML configuration.".to_string(),
            GuiError::TokenNotFound(token) => 
                format!("The token '{}' could not be found in the YAML configuration.", token),
            GuiError::InvalidPreset => 
                "The selected preset is invalid. Please choose a different preset from your YAML configuration.".to_string(),
            GuiError::PresetsNotSet => 
                "No presets have been configured. Please check your YAML configuration.".to_string(),
            GuiError::SelectTokensNotSet => 
                "No tokens have been configured for selection. Please check your YAML configuration.".to_string(),
            GuiError::TokenMustBeSelected(token) => 
                format!("The token '{}' must be selected to proceed.", token),
            GuiError::BindingHasNoPresets(binding) => 
                format!("The binding '{}' does not have any presets configured in the YAML configuration.", binding),
            GuiError::TokenNotInSelectTokens(token) => 
                format!("The token '{}' is not in the list of selectable tokens defined in the YAML configuration.", token),
            GuiError::JsError(msg) => 
                format!("A JavaScript error occurred: {}", msg),
            GuiError::DotrainOrderError(err) => 
                format!("Order configuration error in YAML: {}", err),
            GuiError::ParseGuiConfigSourceError(err) => 
                format!("Failed to parse YAML GUI configuration: {}", err),
            GuiError::IoError(err) => 
                format!("I/O error: {}", err),
            GuiError::BincodeError(err) => 
                format!("Data serialization error: {}", err),
            GuiError::Base64Error(err) => 
                format!("Base64 encoding/decoding error: {}", err),
            GuiError::FromHexError(err) => 
                format!("Invalid hexadecimal value: {}", err),
            GuiError::ReadableClientError(err) => 
                format!("Network client error: {}", err),
            GuiError::DepositError(err) => 
                format!("Deposit error: {}", err),
            GuiError::ParseError(err) => 
                format!("Number parsing error: {}", err),
            GuiError::ReadContractParametersBuilderError(err) => 
                format!("Contract parameter error: {}", err),
            GuiError::UnitsError(err) => 
                format!("Unit conversion error: {}", err),
            GuiError::WritableTransactionExecuteError(err) => 
                format!("Transaction execution error: {}", err),
            GuiError::AddOrderArgsError(err) => 
                format!("Invalid order arguments: {}", err),
            GuiError::ERC20Error(err) => 
                format!("ERC20 token error: {}", err),
            GuiError::SolTypesError(err) => 
                format!("Solidity type error: {}", err),
            GuiError::SerdeWasmBindgenError(err) => 
                format!("Data serialization error: {}", err),
            GuiError::YamlError(err) => 
                err.to_readable_msg(),
        }
    }
}

impl From<GuiError> for JsValue {
    fn from(value: GuiError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<GuiError> for WasmEncodedError {
    fn from(value: GuiError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}
