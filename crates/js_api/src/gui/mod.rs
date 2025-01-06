use alloy::primitives::Address;
use alloy_ethers_typecast::transaction::ReadableClientError;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rain_orderbook_app_settings::{
    config_source::{
        DeployerConfigSource, DeploymentConfigSource, IOString, NetworkConfigSource,
        OrderConfigSource, OrderbookConfigSource, ScenarioConfigSource, TokenConfigSource,
    },
    gui::{
        Gui, GuiConfigSource, GuiDeployment, GuiDeploymentSource, GuiDepositSource,
        GuiFieldDefinition, GuiFieldDefinitionSource, GuiPreset, GuiPresetSource,
        ParseGuiConfigSourceError,
    },
    yaml::{
        default_document, dotrain::DotrainYaml, orderbook::OrderbookYaml, YamlError, YamlParsable,
    },
    Config,
};
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};
use rain_orderbook_common::{
    dotrain::{types::patterns::FRONTMATTER_SEPARATOR, RainDocument},
    dotrain_order::{calldata::DotrainOrderCalldataError, DotrainOrder, DotrainOrderError},
    erc20::{TokenInfo, ERC20},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::RwLock,
};
use std::{io::prelude::*, sync::Arc};
use strict_yaml_rust::StrictYaml;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
#[wasm_bindgen]
pub struct DotrainOrderGui {
    dotrain: String,
    #[serde(skip, default = "default_document")]
    document: Arc<RwLock<StrictYaml>>,
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
        let gui = DotrainYaml::new(
            RainDocument::get_front_matter(&dotrain)
                .ok_or(GuiError::InvalidDotrain)?
                .to_string(),
            true,
        )?
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
        let dotrain_yaml = DotrainYaml::new(
            RainDocument::get_front_matter(&dotrain)
                .ok_or(GuiError::InvalidDotrain)?
                .to_string(),
            true,
        )?;
        let gui = dotrain_yaml.get_gui()?.ok_or(GuiError::GuiConfigNotFound)?;

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
            dotrain,
            document: dotrain_yaml.document,
            selected_deployment: deployment_name,
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            select_tokens,
            onchain_token_info,
        })
    }

    pub async fn get_dotrain_order(&self) -> Result<DotrainOrder, GuiError> {
        let rain_document = RainDocument::create(self.dotrain.clone(), None, None, None);
        let dotrain_body = rain_document.body();

        let orderbook_yaml = OrderbookYaml::from_document(self.document.clone());
        let mut yaml_parts = Vec::new();

        // Networks
        let networks: HashMap<String, NetworkConfigSource> = orderbook_yaml
            .get_network_keys()?
            .iter()
            .map(|network| {
                let value = orderbook_yaml.get_network(network)?;
                Ok((
                    network.clone(),
                    NetworkConfigSource {
                        rpc: value.rpc,
                        chain_id: value.chain_id,
                        label: value.label,
                        network_id: value.network_id,
                        currency: value.currency,
                    },
                ))
            })
            .collect::<Result<_, YamlError>>()?;
        let networks_yaml = serde_yaml::to_string(&HashMap::from([("networks", networks)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        yaml_parts.push(networks_yaml);

        // Tokens
        let tokens: HashMap<String, TokenConfigSource> = orderbook_yaml
            .get_token_keys()?
            .iter()
            .map(|token| {
                let value = orderbook_yaml.get_token(token)?;
                Ok((
                    token.clone(),
                    TokenConfigSource {
                        network: value.network.key.clone(),
                        address: value.address,
                        decimals: value.decimals,
                        label: value.label,
                        symbol: value.symbol,
                    },
                ))
            })
            .collect::<Result<_, YamlError>>()?;
        let tokens_yaml = serde_yaml::to_string(&HashMap::from([("tokens", tokens)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        yaml_parts.push(tokens_yaml);

        // Subgraphs
        let subgraphs: HashMap<String, Url> = orderbook_yaml
            .get_subgraph_keys()?
            .iter()
            .map(|subgraph| {
                let value = orderbook_yaml.get_subgraph(subgraph)?;
                Ok((subgraph.clone(), value.url))
            })
            .collect::<Result<_, YamlError>>()?;
        let subgraphs_yaml = serde_yaml::to_string(&HashMap::from([("subgraphs", subgraphs)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        yaml_parts.push(subgraphs_yaml);

        // Orderbooks
        let orderbooks: HashMap<String, OrderbookConfigSource> = orderbook_yaml
            .get_orderbook_keys()?
            .iter()
            .map(|orderbook| {
                let value = orderbook_yaml.get_orderbook(orderbook)?;
                Ok((
                    orderbook.clone(),
                    OrderbookConfigSource {
                        address: value.address,
                        network: Some(value.network.key.clone()),
                        subgraph: Some(value.subgraph.key.clone()),
                        label: value.label,
                    },
                ))
            })
            .collect::<Result<_, YamlError>>()?;
        let orderbooks_yaml = serde_yaml::to_string(&HashMap::from([("orderbooks", orderbooks)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        yaml_parts.push(orderbooks_yaml);

        // Metaboards
        let metaboards: HashMap<String, Url> = orderbook_yaml
            .get_metaboard_keys()?
            .iter()
            .map(|metaboard| {
                let value = orderbook_yaml.get_metaboard(metaboard)?;
                Ok((metaboard.clone(), value.url))
            })
            .collect::<Result<_, YamlError>>()?;
        let metaboards_yaml = serde_yaml::to_string(&HashMap::from([("metaboards", metaboards)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        yaml_parts.push(metaboards_yaml);

        // Deployers
        let deployers: HashMap<String, DeployerConfigSource> = orderbook_yaml
            .get_deployer_keys()?
            .iter()
            .map(|deployer| {
                let value = orderbook_yaml.get_deployer(deployer)?;
                Ok((
                    deployer.clone(),
                    DeployerConfigSource {
                        address: value.address,
                        network: Some(value.network.key.clone()),
                        label: Some(value.key),
                    },
                ))
            })
            .collect::<Result<_, YamlError>>()?;
        let deployers_yaml = serde_yaml::to_string(&HashMap::from([("deployers", deployers)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        yaml_parts.push(deployers_yaml);

        // Sentry
        let sentry = orderbook_yaml.get_sentry()?;
        let sentry_yaml = serde_yaml::to_string(&HashMap::from([("sentry", sentry)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        yaml_parts.push(sentry_yaml);

        // Combine all orderbook yaml parts
        let yaml_string = yaml_parts
            .into_iter()
            .map(|s| s.trim_start_matches("---\n").to_string())
            .collect::<Vec<_>>()
            .join("\n");

        // Now handle dotrain yaml parts
        let dotrain_yaml = DotrainYaml::from_document(self.document.clone());
        let mut dotrain_yaml_parts = Vec::new();

        // Orders
        let orders: HashMap<String, OrderConfigSource> = dotrain_yaml
            .get_order_keys()?
            .iter()
            .map(|order| {
                let value = dotrain_yaml.get_order(order)?;
                Ok((
                    order.clone(),
                    OrderConfigSource {
                        inputs: value
                            .inputs
                            .iter()
                            .map(|input| IOString {
                                token: input.token.key.clone(),
                                vault_id: input.vault_id,
                            })
                            .collect(),
                        outputs: value
                            .outputs
                            .iter()
                            .map(|output| IOString {
                                token: output.token.key.clone(),
                                vault_id: output.vault_id,
                            })
                            .collect(),
                        deployer: value.deployer.as_ref().map(|d| d.key.clone()),
                        orderbook: value.orderbook.as_ref().map(|o| o.key.clone()),
                    },
                ))
            })
            .collect::<Result<_, YamlError>>()?;
        let orders_yaml = serde_yaml::to_string(&HashMap::from([("orders", orders)]))
            .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        dotrain_yaml_parts.push(orders_yaml);

        // Scenarios
        {
            let mut scenarios = HashMap::<String, ScenarioConfigSource>::new();

            let mut scenario_keys = dotrain_yaml.get_scenario_keys()?;
            scenario_keys.sort_by_key(|k| {
                let path_len = k.split('.').count();
                std::cmp::Reverse(path_len)
            });

            fn get_parent_key(key: &str) -> Option<String> {
                let mut parts = key.rsplitn(2, '.');
                parts.next()?;
                parts.next().map(|s| s.to_string())
            }

            for key in scenario_keys.clone() {
                let scenario = dotrain_yaml.get_scenario(&key)?;

                let config_source = ScenarioConfigSource {
                    bindings: scenario.bindings.clone(),
                    runs: scenario.runs,
                    blocks: scenario.blocks.clone(),
                    deployer: Some(scenario.deployer.key.clone()),
                    scenarios: None,
                };

                scenarios.insert(key.clone(), config_source);
            }

            for key in scenario_keys.iter() {
                if let Some(parent_key) = get_parent_key(key) {
                    let children: HashMap<String, ScenarioConfigSource> = scenarios
                        .iter()
                        .filter(|(k, _)| get_parent_key(k) == Some(parent_key.clone()))
                        .map(|(k, v)| {
                            let child_name = k.split('.').last().unwrap().to_string();
                            (child_name, v.clone())
                        })
                        .collect();

                    if !children.is_empty() {
                        if let Some(parent_scenario) = scenarios.get_mut(&parent_key) {
                            parent_scenario.scenarios = Some(children);
                        }
                    }
                }
            }

            let root_scenarios: HashMap<String, ScenarioConfigSource> = scenarios
                .into_iter()
                .filter(|(k, _)| !k.contains('.'))
                .collect();

            let scenarios_yaml =
                serde_yaml::to_string(&HashMap::from([("scenarios", root_scenarios)]))
                    .map_err(|err| GuiError::SerializationError(err.to_string()))?;
            dotrain_yaml_parts.push(scenarios_yaml);
        }

        // Deployments
        let deployments: HashMap<String, DeploymentConfigSource> = dotrain_yaml
            .get_deployment_keys()?
            .iter()
            .map(|deployment| {
                let value = dotrain_yaml.get_deployment(deployment)?;
                Ok((
                    deployment.clone(),
                    DeploymentConfigSource {
                        scenario: value.scenario.key.clone(),
                        order: value.order.key.clone(),
                    },
                ))
            })
            .collect::<Result<_, YamlError>>()?;
        let deployments_yaml =
            serde_yaml::to_string(&HashMap::from([("deployments", deployments)]))
                .map_err(|err| GuiError::SerializationError(err.to_string()))?;
        dotrain_yaml_parts.push(deployments_yaml);

        // GUI
        if let Some(gui) = dotrain_yaml.get_gui()? {
            let source = GuiConfigSource {
                name: gui.name,
                description: gui.description,
                deployments: gui
                    .deployments
                    .iter()
                    .map(|(name, deployment)| {
                        let source = GuiDeploymentSource {
                            name: deployment.name.clone(),
                            description: deployment.description.clone(),
                            deposits: deployment
                                .deposits
                                .iter()
                                .map(|deposit| GuiDepositSource {
                                    token: deposit.token.key.clone(),
                                    presets: deposit.presets.clone(),
                                })
                                .collect(),
                            fields: deployment
                                .fields
                                .iter()
                                .map(|field| GuiFieldDefinitionSource {
                                    binding: field.binding.clone(),
                                    name: field.name.clone(),
                                    description: field.description.clone(),
                                    presets: field.presets.as_ref().map(|presets| {
                                        presets
                                            .iter()
                                            .map(|preset| GuiPresetSource {
                                                name: preset.name.clone(),
                                                value: preset.value.clone(),
                                            })
                                            .collect()
                                    }),
                                })
                                .collect(),
                            select_tokens: deployment
                                .select_tokens
                                .as_ref()
                                .map(|tokens| tokens.iter().map(|token| token.clone()).collect()),
                        };
                        (name.clone(), source)
                    })
                    .collect(),
            };

            let gui_yaml = serde_yaml::to_string(&HashMap::from([("gui", source)]))
                .map_err(|err| GuiError::SerializationError(err.to_string()))?;
            dotrain_yaml_parts.push(gui_yaml);
        }

        // Combine all dotrain yaml parts
        let dotrain_yaml_string = dotrain_yaml_parts
            .into_iter()
            .map(|s| s.trim_start_matches("---\n").to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let dotrain = format!(
            "{}\n\n{}\n\n{}\n{}",
            yaml_string, dotrain_yaml_string, FRONTMATTER_SEPARATOR, dotrain_body
        );
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;

        Ok(dotrain_order)
    }

    #[wasm_bindgen(js_name = "getDotrainConfig")]
    pub async fn get_dotrain_config(&self) -> Result<Config, GuiError> {
        let dotrain_order = self.get_dotrain_order().await?;
        Ok(dotrain_order.config().clone())
    }

    #[wasm_bindgen(js_name = "getGuiConfig")]
    pub fn get_gui_config(&self) -> Result<Gui, GuiError> {
        let gui = DotrainYaml::from_document(self.document.clone())
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
impl PartialEq for DotrainOrderGui {
    fn eq(&self, other: &Self) -> bool {
        self.dotrain == other.dotrain
            && self.selected_deployment == other.selected_deployment
            && self.field_values == other.field_values
            && self.deposits == other.deposits
            && self.select_tokens == other.select_tokens
            && self.onchain_token_info == other.onchain_token_info
    }
}
impl Default for DotrainOrderGui {
    fn default() -> Self {
        Self {
            dotrain: "".to_string(),
            document: default_document(),
            selected_deployment: "".to_string(),
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            select_tokens: None,
            onchain_token_info: BTreeMap::new(),
        }
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
    #[error("Invalid dotrain")]
    InvalidDotrain,
    #[error("Serialization error: {0}")]
    SerializationError(String),
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
