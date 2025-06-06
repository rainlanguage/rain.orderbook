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
impl Default for DotrainOrderGui {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_export]
impl DotrainOrderGui {
    #[wasm_export(js_name = "getDeploymentKeys", unchecked_return_type = "string[]")]
    pub async fn get_deployment_keys(dotrain: String) -> Result<Vec<String>, GuiError> {
        let mut dotrain_order = DotrainOrder::new();
        dotrain_order.initialize(dotrain.clone(), None).await?;
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
        let mut dotrain_order = DotrainOrder::new();
        dotrain_order.initialize(dotrain.clone(), None).await?;

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
        if !GuiCfg::check_gui_key_exists(self.dotrain_order.dotrain_yaml().documents.clone())? {
            return Err(GuiError::GuiConfigNotFound);
        }
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
        let mut dotrain_order = DotrainOrder::new();
        dotrain_order.initialize(dotrain.clone(), None).await?;

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
        let mut dotrain_order = DotrainOrder::new();
        dotrain_order.initialize(dotrain.clone(), None).await?;

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

    #[wasm_export(
        js_name = "getCurrentDeploymentDetails",
        unchecked_return_type = "NameAndDescriptionCfg"
    )]
    pub fn get_current_deployment_details(&self) -> Result<NameAndDescriptionCfg, GuiError> {
        let deployment_details =
            GuiCfg::parse_deployment_details(self.dotrain_order.dotrain_yaml().documents.clone())?;
        Ok(deployment_details
            .get(&self.selected_deployment)
            .ok_or(GuiError::DeploymentNotFound(
                self.selected_deployment.clone(),
            ))?
            .clone())
    }

    #[wasm_export(js_name = "generateDotrainText", unchecked_return_type = "string")]
    pub fn generate_dotrain_text(&self) -> Result<String, GuiError> {
        let rain_document = RainDocument::create(self.dotrain_order.dotrain()?, None, None, None);
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

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order.initialize(dotrain.clone(), None).await?;

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
    #[error("Missing deposit token for current deployment: {0}")]
    MissingDepositToken(String),
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
            GuiError::MissingDepositToken(deployment) =>
                format!("A deposit for token is required but has not been set for deployment '{}'.", deployment),
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
            GuiError::YamlError(err) => format!("YAML configuration error: {}", err),
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

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_app_settings::yaml::FieldErrorKind;
    use wasm_bindgen_test::wasm_bindgen_test;

    pub fn get_yaml() -> String {
        format!(
            r#"
version: {spec_version}
gui:
  name: Fixed limit
  description: Fixed limit order strategy
  short-description: Buy WETH with USDC on Base.
  deployments:
    some-deployment:
      name: Buy WETH with USDC on Base.
      description: Buy WETH with USDC for fixed price on Base network.
      short-description: Buy WETH with USDC on Base.
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
            - "10"
            - "100"
            - "1000"
            - "10000"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0x1234567890abcdef1234567890abcdef12345678"
            - name: Preset 2
              value: "false"
            - name: Preset 3
              value: "some-string"
          default: some-default-value
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          presets:
            - value: "99.2"
            - value: "582.1"
            - value: "648.239"
          show-custom-field: true
    other-deployment:
      name: Test test
      description: Test test test
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
    select-token-deployment:
      name: Select token deployment
      description: Select token deployment description
      deposits:
        - token: token3
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
      select-tokens:
        - key: token3
          name: Token 3
          description: Token 3 description
        - key: token4
          name: Token 4
          description: Token 4 description
networks:
    some-network:
        rpc: http://localhost:8085/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH
subgraphs:
    some-sg: https://www.some-sg.com
metaboards:
    test: https://metaboard.com
deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: Token 1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2
scenarios:
    some-scenario:
        deployer: some-deployer
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300
orders:
    some-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token2
          vault-id: 1
      deployer: some-deployer
      orderbook: some-orderbook
    other-order:
      inputs:
        - token: token1
      outputs:
        - token: token1
      deployer: some-deployer
      orderbook: some-orderbook
deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    other-deployment:
        scenario: some-scenario.sub-scenario
        order: other-order
    select-token-deployment:
        scenario: some-scenario
        order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        )
    }

    pub async fn initialize_gui(deployment_name: Option<String>) -> DotrainOrderGui {
        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(
            get_yaml(),
            deployment_name.unwrap_or("some-deployment".to_string()),
            None,
        )
        .await
        .unwrap();
        gui
    }

    pub async fn initialize_gui_with_select_tokens() -> DotrainOrderGui {
        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(get_yaml(), "select-token-deployment".to_string(), None)
            .await
            .unwrap();
        gui
    }

    #[wasm_bindgen_test]
    async fn test_get_deployment_keys() {
        let deployment_keys = DotrainOrderGui::get_deployment_keys(get_yaml())
            .await
            .unwrap();
        assert_eq!(
            deployment_keys,
            vec![
                "some-deployment",
                "other-deployment",
                "select-token-deployment"
            ]
        );
    }

    #[wasm_bindgen_test]
    async fn test_choose_deployment() {
        let mut gui = DotrainOrderGui::new();

        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        let err = gui
            .choose_deployment(get_yaml(), "invalid-deployment".to_string(), None)
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DeploymentNotFound("invalid-deployment".to_string()).to_string()
        );
        assert_eq!(err.to_readable_msg(), "The deployment 'invalid-deployment' could not be found. Please select a valid deployment from your YAML configuration.");
    }

    #[wasm_bindgen_test]
    async fn test_get_gui_config() {
        let mut gui = DotrainOrderGui::new();

        let err = gui.get_gui_config().unwrap_err();
        assert_eq!(err.to_string(), GuiError::GuiConfigNotFound.to_string());
        assert_eq!(
            err.to_readable_msg(),
            "The GUI configuration could not be found. Please check your YAML configuration file."
        );

        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        let gui_config = gui.get_gui_config().unwrap();
        assert_eq!(gui_config.name, "Fixed limit".to_string());
        assert_eq!(
            gui_config.description,
            "Fixed limit order strategy".to_string()
        );
        assert_eq!(gui_config.deployments.len(), 1);
        let deployment = gui_config.deployments.get("some-deployment").unwrap();
        assert_eq!(deployment.name, "Buy WETH with USDC on Base.".to_string());
        assert_eq!(
            deployment.description,
            "Buy WETH with USDC for fixed price on Base network.".to_string()
        );
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = deployment.deposits[0].clone();
        assert!(deposit.token.is_some());
        assert_eq!(deposit.token.unwrap().key, "token1");
        assert_eq!(
            deposit.presets,
            Some(vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ])
        );
        assert_eq!(deployment.fields.len(), 2);
        let field = deployment.fields[0].clone();
        assert_eq!(field.name, "Field 1 name");
        assert_eq!(field.description, Some("Field 1 description".to_string()));
        assert_eq!(
            field.presets,
            Some(vec![
                GuiPresetCfg {
                    id: "0".to_string(),
                    name: Some("Preset 1".to_string()),
                    value: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                },
                GuiPresetCfg {
                    id: "1".to_string(),
                    name: Some("Preset 2".to_string()),
                    value: "false".to_string(),
                },
                GuiPresetCfg {
                    id: "2".to_string(),
                    name: Some("Preset 3".to_string()),
                    value: "some-string".to_string(),
                },
            ])
        );
        let field = deployment.fields[1].clone();
        assert_eq!(field.name, "Field 2 name");
        assert_eq!(field.description, Some("Field 2 description".to_string()));
        assert_eq!(field.show_custom_field, Some(true));
        assert_eq!(
            field.presets,
            Some(vec![
                GuiPresetCfg {
                    id: "0".to_string(),
                    name: None,
                    value: "99.2".to_string(),
                },
                GuiPresetCfg {
                    id: "1".to_string(),
                    name: None,
                    value: "582.1".to_string(),
                },
                GuiPresetCfg {
                    id: "2".to_string(),
                    name: None,
                    value: "648.239".to_string(),
                },
            ])
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_current_deployment() {
        let mut gui = DotrainOrderGui::new();

        let err = gui.get_current_deployment().unwrap_err();
        assert_eq!(err.to_string(), GuiError::GuiConfigNotFound.to_string());
        assert_eq!(
            err.to_readable_msg(),
            "The GUI configuration could not be found. Please check your YAML configuration file."
        );

        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        let deployment = gui.get_current_deployment().unwrap();
        assert_eq!(deployment.name, "Buy WETH with USDC on Base.".to_string());
        assert_eq!(
            deployment.description,
            "Buy WETH with USDC for fixed price on Base network.".to_string()
        );
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = deployment.deposits[0].clone();
        assert!(deposit.token.is_some());
        assert_eq!(deposit.token.unwrap().key, "token1");
        assert_eq!(
            deposit.presets,
            Some(vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ])
        );
        assert_eq!(deployment.fields.len(), 2);
        let field = deployment.fields[0].clone();
        assert_eq!(field.name, "Field 1 name");
        assert_eq!(field.description, Some("Field 1 description".to_string()));
        assert_eq!(
            field.presets,
            Some(vec![
                GuiPresetCfg {
                    id: "0".to_string(),
                    name: Some("Preset 1".to_string()),
                    value: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                },
                GuiPresetCfg {
                    id: "1".to_string(),
                    name: Some("Preset 2".to_string()),
                    value: "false".to_string(),
                },
                GuiPresetCfg {
                    id: "2".to_string(),
                    name: Some("Preset 3".to_string()),
                    value: "some-string".to_string(),
                },
            ])
        );
        let field = deployment.fields[1].clone();
        assert_eq!(field.name, "Field 2 name");
        assert_eq!(field.description, Some("Field 2 description".to_string()));
        assert_eq!(field.show_custom_field, Some(true));
        assert_eq!(
            field.presets,
            Some(vec![
                GuiPresetCfg {
                    id: "0".to_string(),
                    name: None,
                    value: "99.2".to_string(),
                },
                GuiPresetCfg {
                    id: "1".to_string(),
                    name: None,
                    value: "582.1".to_string(),
                },
                GuiPresetCfg {
                    id: "2".to_string(),
                    name: None,
                    value: "648.239".to_string(),
                },
            ])
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_token_info_local() {
        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        let token1_info = gui.get_token_info("token1".to_string()).await.unwrap();
        assert_eq!(
            token1_info.address.to_string(),
            "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"
        );
        assert_eq!(token1_info.decimals, 6);
        assert_eq!(token1_info.name, "Token 1");
        assert_eq!(token1_info.symbol, "T1");

        let token2_info = gui.get_token_info("token2".to_string()).await.unwrap();
        assert_eq!(
            token2_info.address.to_string(),
            "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"
        );
        assert_eq!(token2_info.decimals, 18);
        assert_eq!(token2_info.name, "Token 2");
        assert_eq!(token2_info.symbol, "T2");

        let err = gui
            .get_token_info("invalid-token".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::KeyNotFound("invalid-token".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Key 'invalid-token' not found"
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_all_token_infos_local() {
        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        let token_infos = gui.get_all_token_infos().await.unwrap();
        assert_eq!(token_infos.len(), 2);
        assert_eq!(
            token_infos[0].address.to_string(),
            "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"
        );
        assert_eq!(token_infos[0].decimals, 6);
        assert_eq!(token_infos[0].name, "Token 1");
        assert_eq!(token_infos[0].symbol, "T1");
        assert_eq!(
            token_infos[1].address.to_string(),
            "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"
        );
        assert_eq!(token_infos[1].decimals, 18);
        assert_eq!(token_infos[1].name, "Token 2");
        assert_eq!(token_infos[1].symbol, "T2");
    }

    #[wasm_bindgen_test]
    async fn test_get_strategy_details() {
        let strategy_details = DotrainOrderGui::get_strategy_details(get_yaml())
            .await
            .unwrap();
        assert_eq!(strategy_details.name, "Fixed limit");
        assert_eq!(strategy_details.description, "Fixed limit order strategy");
        assert_eq!(
            strategy_details.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_strategy_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "gui".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'name' in gui"
        );

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    name: Test name
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_strategy_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "gui".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'description' in gui"
        );

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    name: Test name
    description: Test description
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_strategy_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("short-description".to_string()),
                location: "gui".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'short-description' in gui"
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_deployment_details() {
        let deployment_details = DotrainOrderGui::get_deployment_details(get_yaml())
            .await
            .unwrap();
        assert_eq!(deployment_details.len(), 3);
        let deployment_detail = deployment_details.get("some-deployment").unwrap();
        assert_eq!(deployment_detail.name, "Buy WETH with USDC on Base.");
        assert_eq!(
            deployment_detail.description,
            "Buy WETH with USDC for fixed price on Base network."
        );
        assert_eq!(
            deployment_detail.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );
        let deployment_detail = deployment_details.get("other-deployment").unwrap();
        assert_eq!(deployment_detail.name, "Test test");
        assert_eq!(deployment_detail.description, "Test test test");
        assert_eq!(deployment_detail.short_description, None);
        let deployment_detail = deployment_details.get("select-token-deployment").unwrap();
        assert_eq!(deployment_detail.name, "Select token deployment");
        assert_eq!(
            deployment_detail.description,
            "Select token deployment description"
        );
        assert_eq!(deployment_detail.short_description, None);

        let yaml = format!(
            r#"
version: {spec_version}
test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let details = DotrainOrderGui::get_deployment_details(yaml.to_string())
            .await
            .unwrap();
        assert_eq!(details.len(), 0);

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_deployment_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "gui".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'deployments' in gui"
        );

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    deployments: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_deployment_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string(),
                },
                location: "gui".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Field 'deployments' must be a map in gui"
        );

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    deployments:
        - test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_deployment_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string(),
                },
                location: "gui".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Field 'deployments' must be a map in gui"
        );

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    deployments:
        test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_deployment_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "gui deployment 'test'".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'name' in gui deployment 'test'"
        );

        let yaml = format!(
            r#"
version: {spec_version}
gui:
    deployments:
        test:
            name: Test name
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = DotrainOrderGui::get_deployment_details(yaml.to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "gui deployment 'test'".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'description' in gui deployment 'test'"
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_deployment_detail() {
        let deployment_detail =
            DotrainOrderGui::get_deployment_detail(get_yaml(), "some-deployment".to_string())
                .await
                .unwrap();
        assert_eq!(deployment_detail.name, "Buy WETH with USDC on Base.");
        assert_eq!(
            deployment_detail.description,
            "Buy WETH with USDC for fixed price on Base network."
        );
        assert_eq!(
            deployment_detail.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_current_deployment_detail() {
        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        let deployment_detail = gui.get_current_deployment_details().unwrap();
        assert_eq!(deployment_detail.name, "Buy WETH with USDC on Base.");
        assert_eq!(
            deployment_detail.description,
            "Buy WETH with USDC for fixed price on Base network."
        );
        assert_eq!(
            deployment_detail.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );
    }

    #[wasm_bindgen_test]
    async fn test_generate_dotrain_text() {
        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();
        let original_current_deployment = gui.get_current_deployment_details().unwrap();

        let dotrain_text = gui.generate_dotrain_text().unwrap();
        gui.choose_deployment(dotrain_text, "some-deployment".to_string(), None)
            .await
            .unwrap();
        let new_current_deployment = gui.get_current_deployment_details().unwrap();

        assert_eq!(new_current_deployment, original_current_deployment);
    }

    #[wasm_bindgen_test]
    async fn test_get_composed_rainlang() {
        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(get_yaml(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        let rainlang = gui.get_composed_rainlang().await.unwrap();
        let expected_rainlang =
            "/* 0. calculate-io */ \n_ _: 0 0;\n\n/* 1. handle-io */ \n:;".to_string();
        assert_eq!(rainlang, expected_rainlang);
    }

    #[cfg(not(target_family = "wasm"))]
    mod select_token_tests {
        use super::*;
        use alloy_ethers_typecast::rpc::Response;
        use httpmock::MockServer;

        pub const SELECT_TOKEN_YAML: &str = r#"
gui:
    name: Fixed limit
    description: Fixed limit order strategy
    short-description: Buy WETH with USDC on Base.
    deployments:
        some-deployment:
            name: Select token deployment
            description: Select token deployment description
            deposits:
            - token: token3
              min: 0
              presets:
                - "0"
            fields:
            - binding: binding-1
              name: Field 1 name
              description: Field 1 description
              presets:
                - name: Preset 1
                  value: "0"
            - binding: binding-2
              name: Field 2 name
              description: Field 2 description
              min: 100
              presets:
                - value: "0"
            select-tokens:
            - key: token3
              name: Token 3
              description: Token 3 description
subgraphs:
    some-sg: https://www.some-sg.com
metaboards:
    test: https://metaboard.com
deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
scenarios:
    some-scenario:
        deployer: some-deployer
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300
orders:
    some-order:
        deployer: some-deployer
        inputs:
            - token: token3
        outputs:
            - token: token3
deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    normal-deployment:
        scenario: some-scenario
        order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#;

        #[tokio::test]
        async fn test_get_token_info_remote() {
            let server = MockServer::start_async().await;
            let yaml = format!(
                r#"
networks:
    some-network:
        rpc: {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
{yaml}
"#,
                yaml = SELECT_TOKEN_YAML,
                rpc_url = server.url("/rpc")
            );

            server.mock(|when, then| {
                        when.method("POST").path("/rpc").body_contains("0x82ad56cb");
                        then.body(Response::new_success(1, "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000").to_json_string().unwrap());
                    });

            let mut gui = DotrainOrderGui::new();
            gui.choose_deployment(yaml.to_string(), "some-deployment".to_string(), None)
                .await
                .unwrap();

            let err = gui.get_token_info("token3".to_string()).await.unwrap_err();
            assert_eq!(
                err.to_string(),
                YamlError::Field {
                    kind: FieldErrorKind::Missing("tokens".to_string()),
                    location: "root".to_string(),
                }
                .to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "YAML configuration error: Missing required field 'tokens' in root"
            );

            gui.save_select_token(
                "token3".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
            )
            .await
            .unwrap();

            let token_info = gui.get_token_info("token3".to_string()).await.unwrap();
            assert_eq!(
                token_info.address.to_string(),
                "0x0000000000000000000000000000000000000001"
            );
            assert_eq!(token_info.decimals, 6);
            assert_eq!(token_info.name, "Token 1");
            assert_eq!(token_info.symbol, "T1");

            let token_infos = gui.get_all_token_infos().await.unwrap();
            assert_eq!(token_infos.len(), 1);
            assert_eq!(
                token_infos[0].address.to_string(),
                "0x0000000000000000000000000000000000000001"
            );
            assert_eq!(token_infos[0].decimals, 6);
            assert_eq!(token_infos[0].name, "Token 1");
            assert_eq!(token_infos[0].symbol, "T1");
        }
    }
}
