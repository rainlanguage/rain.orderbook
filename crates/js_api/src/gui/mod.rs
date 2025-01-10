use alloy::primitives::Address;
use alloy_ethers_typecast::transaction::ReadableClientError;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rain_orderbook_app_settings::{
    gui::{Gui, GuiDeployment, GuiFieldDefinition, GuiPreset, ParseGuiConfigSourceError},
    yaml::YamlError,
};
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};
use rain_orderbook_common::{
    dotrain_order::{calldata::DotrainOrderCalldataError, DotrainOrder, DotrainOrderError},
    erc20::{TokenInfo, ERC20},
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
pub struct AvailableDeployments(Vec<GuiDeployment>);
impl_all_wasm_traits!(AvailableDeployments);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TokenInfos(#[tsify(type = "Map<string, TokenInfo>")] BTreeMap<Address, TokenInfo>);
impl_all_wasm_traits!(TokenInfos);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct DotrainOrderGui {
    dotrain_order: DotrainOrder,
    selected_deployment: String,
    field_values: BTreeMap<String, field_values::PairValue>,
    deposits: BTreeMap<String, field_values::PairValue>,
    select_tokens: Option<BTreeMap<String, Address>>,
    onchain_token_info: BTreeMap<Address, TokenInfo>,
}
#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "getAvailableDeployments")]
    pub async fn get_available_deployments(
        dotrain: String,
    ) -> Result<AvailableDeployments, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;
        let gui = dotrain_order
            .dotrain_yaml()
            .get_gui()?
            .ok_or(GuiError::GuiConfigNotFound)?;
        Ok(AvailableDeployments(
            gui.deployments.values().cloned().collect(),
        ))
    }

    #[wasm_bindgen(js_name = "chooseDeployment")]
    pub async fn choose_deployment(
        dotrain: String,
        deployment_name: String,
        multicall_address: Option<String>,
    ) -> Result<DotrainOrderGui, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;

        let gui = dotrain_order
            .dotrain_yaml()
            .get_gui()?
            .ok_or(GuiError::GuiConfigNotFound)?;

        let (_, gui_deployment) = gui
            .deployments
            .into_iter()
            .find(|(name, _)| name == &deployment_name)
            .ok_or(GuiError::DeploymentNotFound(deployment_name.clone()))?;

        let select_tokens = gui_deployment.select_tokens.clone().map(|tokens| {
            tokens
                .iter()
                .map(|token: &String| (token.clone(), Address::ZERO))
                .collect::<BTreeMap<String, Address>>()
        });

        let rpc_url = gui_deployment
            .deployment
            .order
            .orderbook
            .clone()
            .ok_or(GuiError::OrderbookNotFound)?
            .network
            .rpc
            .clone();
        let mut onchain_token_info: BTreeMap<Address, TokenInfo> = BTreeMap::new();
        for token in gui_deployment.deposits.iter() {
            if onchain_token_info.contains_key(&token.token.address) {
                continue;
            }

            if let Some(select_tokens) = &select_tokens {
                if select_tokens.contains_key(&token.token.key) {
                    continue;
                }
            }

            let erc20 = ERC20::new(rpc_url.clone(), token.token.address);
            let token_info = erc20.token_info(multicall_address.clone()).await?;
            onchain_token_info.insert(token.token.address, token_info);
        }

        Ok(Self {
            dotrain_order,
            selected_deployment: deployment_name.clone(),
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            select_tokens,
            onchain_token_info,
        })
    }

    #[wasm_bindgen(js_name = "getGuiConfig")]
    pub fn get_gui_config(&self) -> Result<Gui, GuiError> {
        let gui = self
            .dotrain_order
            .dotrain_yaml()
            .get_gui()?
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

    /// Get all token infos in input and output vaults
    ///
    /// Returns a map of token address to [`TokenInfo`]
    #[wasm_bindgen(js_name = "getTokenInfos")]
    pub fn get_token_infos(&self) -> Result<TokenInfos, GuiError> {
        Ok(TokenInfos(self.onchain_token_info.clone()))
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
    #[error("Deserialized config mismatch")]
    DeserializedConfigMismatch,
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
