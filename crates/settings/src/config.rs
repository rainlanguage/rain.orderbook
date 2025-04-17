use crate::yaml::dotrain::DotrainYaml;
use crate::yaml::orderbook::OrderbookYaml;
use crate::yaml::{YamlError, YamlParsable};
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use subgraph::SubgraphCfg;
use thiserror::Error;
use url::Url;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{
    impl_wasm_traits, prelude::*, serialize_hashmap_as_object, serialize_opt_hashmap_as_object,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct Config {
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, NetworkCfg>")
    )]
    networks: HashMap<String, Arc<NetworkCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, SubgraphCfg>")
    )]
    subgraphs: HashMap<String, Arc<SubgraphCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, string>")
    )]
    metaboards: HashMap<String, Arc<Url>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, OrderbookCfg>")
    )]
    orderbooks: HashMap<String, Arc<OrderbookCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, TokenCfg>")
    )]
    tokens: HashMap<String, Arc<TokenCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, DeployerCfg>")
    )]
    deployers: HashMap<String, Arc<DeployerCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, OrderCfg>")
    )]
    orders: HashMap<String, Arc<OrderCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ScenarioCfg>")
    )]
    scenarios: HashMap<String, Arc<ScenarioCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ChartCfg>")
    )]
    charts: HashMap<String, Arc<ChartCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, DeploymentCfg>")
    )]
    deployments: HashMap<String, Arc<DeploymentCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    sentry: Option<bool>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    raindex_version: Option<String>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_opt_hashmap_as_object"),
        tsify(type = "Record<string, string>", optional)
    )]
    accounts: Option<HashMap<String, Arc<String>>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    gui: Option<GuiCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(Config);

#[derive(Error, Debug)]
pub enum ParseConfigError {
    #[error(transparent)]
    ParseNetworkCfgError(#[from] ParseNetworkCfgError),
    #[error(transparent)]
    ParseOrderbookCfgError(#[from] ParseOrderbookCfgError),
    #[error(transparent)]
    ParseTokenCfgError(#[from] ParseTokenCfgError),
    #[error(transparent)]
    ParseOrderCfgError(#[from] ParseOrderCfgError),
    #[error(transparent)]
    ParseDeployerCfgError(#[from] ParseDeployerCfgError),
    #[error(transparent)]
    ParseScenarioCfgError(#[from] ParseScenarioCfgError),
    #[error(transparent)]
    ParseDeploymentCfgError(#[from] ParseDeploymentCfgError),
    #[error(transparent)]
    ParseGuiCfgError(#[from] ParseGuiCfgError),
    #[error(transparent)]
    YamlError(#[from] YamlError),
    #[error("Network not found: {0}")]
    NetworkNotFound(String),
    #[error("Subgraph not found: {0}")]
    SubgraphNotFound(String),
    #[error("Metaboard not found: {0}")]
    MetaboardNotFound(String),
    #[error("Orderbook not found: {0}")]
    OrderbookNotFound(String),
    #[error("Token not found: {0}")]
    OrderNotFound(String),
    #[error("Deployer not found: {0}")]
    TokenNotFound(String),
    #[error("Order not found: {0}")]
    DeployerNotFound(String),
    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),
    #[error("Chart not found: {0}")]
    ChartNotFound(String),
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),
}

impl Config {
    pub fn try_from_settings(settings: Vec<String>) -> Result<Self, ParseConfigError> {
        let dotrain_yaml = DotrainYaml::new(settings.clone(), false)?;
        let orderbook_yaml = OrderbookYaml::new(settings, false)?;

        let networks = orderbook_yaml
            .get_networks()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let subgraphs = orderbook_yaml
            .get_subgraphs()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let metaboards = orderbook_yaml
            .get_metaboards()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v.url)))
            .collect::<HashMap<_, _>>();
        let orderbooks = orderbook_yaml
            .get_orderbooks()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let tokens = orderbook_yaml
            .get_tokens()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let deployers = orderbook_yaml
            .get_deployers()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let orders = dotrain_yaml
            .get_orders()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let scenarios = dotrain_yaml
            .get_scenarios()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let deployments = dotrain_yaml
            .get_deployments()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let charts = dotrain_yaml
            .get_charts()?
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let sentry = orderbook_yaml.get_sentry()?;
        let raindex_version = orderbook_yaml.get_raindex_version()?;
        let gui = dotrain_yaml.get_gui(None)?;

        let config = Config {
            networks,
            subgraphs,
            metaboards,
            orderbooks,
            tokens,
            deployers,
            orders,
            scenarios,
            charts,
            deployments,
            sentry,
            raindex_version,
            // TODO: add accounts
            accounts: None,
            gui,
        };
        Ok(config)
    }

    pub fn get_networks(&self) -> &HashMap<String, Arc<NetworkCfg>> {
        &self.networks
    }
    pub fn get_network(&self, key: &str) -> Result<&Arc<NetworkCfg>, ParseConfigError> {
        self.networks
            .get(key)
            .ok_or(ParseConfigError::NetworkNotFound(key.to_string()))
    }

    pub fn get_subgraphs(&self) -> &HashMap<String, Arc<SubgraphCfg>> {
        &self.subgraphs
    }
    pub fn get_subgraph(&self, key: &str) -> Result<&Arc<SubgraphCfg>, ParseConfigError> {
        self.subgraphs
            .get(key)
            .ok_or(ParseConfigError::SubgraphNotFound(key.to_string()))
    }

    pub fn get_metaboards(&self) -> &HashMap<String, Arc<Url>> {
        &self.metaboards
    }
    pub fn get_metaboard(&self, key: &str) -> Result<&Arc<Url>, ParseConfigError> {
        self.metaboards
            .get(key)
            .ok_or(ParseConfigError::MetaboardNotFound(key.to_string()))
    }

    pub fn get_orderbooks(&self) -> &HashMap<String, Arc<OrderbookCfg>> {
        &self.orderbooks
    }
    pub fn get_orderbook(&self, key: &str) -> Result<&Arc<OrderbookCfg>, ParseConfigError> {
        self.orderbooks
            .get(key)
            .ok_or(ParseConfigError::OrderbookNotFound(key.to_string()))
    }

    pub fn get_tokens(&self) -> &HashMap<String, Arc<TokenCfg>> {
        &self.tokens
    }
    pub fn get_token(&self, key: &str) -> Result<&Arc<TokenCfg>, ParseConfigError> {
        self.tokens
            .get(key)
            .ok_or(ParseConfigError::TokenNotFound(key.to_string()))
    }

    pub fn get_deployers(&self) -> &HashMap<String, Arc<DeployerCfg>> {
        &self.deployers
    }
    pub fn get_deployer(&self, key: &str) -> Result<&Arc<DeployerCfg>, ParseConfigError> {
        self.deployers
            .get(key)
            .ok_or(ParseConfigError::DeployerNotFound(key.to_string()))
    }

    pub fn get_orders(&self) -> &HashMap<String, Arc<OrderCfg>> {
        &self.orders
    }
    pub fn get_order(&self, key: &str) -> Result<&Arc<OrderCfg>, ParseConfigError> {
        self.orders
            .get(key)
            .ok_or(ParseConfigError::OrderNotFound(key.to_string()))
    }

    pub fn get_scenarios(&self) -> &HashMap<String, Arc<ScenarioCfg>> {
        &self.scenarios
    }
    pub fn get_scenario(&self, key: &str) -> Result<&Arc<ScenarioCfg>, ParseConfigError> {
        self.scenarios
            .get(key)
            .ok_or(ParseConfigError::ScenarioNotFound(key.to_string()))
    }

    pub fn get_deployments(&self) -> &HashMap<String, Arc<DeploymentCfg>> {
        &self.deployments
    }
    pub fn get_deployment(&self, key: &str) -> Result<&Arc<DeploymentCfg>, ParseConfigError> {
        self.deployments
            .get(key)
            .ok_or(ParseConfigError::DeploymentNotFound(key.to_string()))
    }

    pub fn get_charts(&self) -> &HashMap<String, Arc<ChartCfg>> {
        &self.charts
    }
    pub fn get_chart(&self, key: &str) -> Result<&Arc<ChartCfg>, ParseConfigError> {
        self.charts
            .get(key)
            .ok_or(ParseConfigError::ChartNotFound(key.to_string()))
    }

    pub fn get_gui(&self) -> &Option<GuiCfg> {
        &self.gui
    }

    pub fn get_sentry(&self) -> &Option<bool> {
        &self.sentry
    }

    pub fn get_raindex_version(&self) -> &Option<String> {
        &self.raindex_version
    }

    pub fn get_accounts(&self) -> &Option<HashMap<String, Arc<String>>> {
        &self.accounts
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    const ORDERBOOK_YAML: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
        testnet:
            rpc: https://testnet.infura.io
            chain-id: 1337
    tokens:
        token1:
            network: mainnet
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
        token2:
            network: mainnet
            address: 0x0000000000000000000000000000000000000002
            decimals: 6
            label: USD Coin
            symbol: USDC
    deployers:
        scenario1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
        deployer2:
            address: 0x0000000000000000000000000000000000000003
            network: testnet
    "#;
    const DOTRAIN_YAML: &str = r#"
    orders:
        order1:
            inputs:
                - token: token1
                  vault-id: 1
            outputs:
                - token: token2
                  vault-id: 2
    scenarios:
        scenario1:
            bindings:
                key1: value1
            scenarios:
                scenario2:
                    bindings:
                        key2: value2
                    scenarios:
                        runs: 10
    deployments:
        deployment1:
            order: order1
            scenario: scenario1.scenario2
        deployment2:
            order: order1
            scenario: scenario1
    gui:
        name: Test gui
        description: Test description
        short-description: Test short description
        deployments:
            deployment1:
                name: Test deployment
                description: Test description
                deposits:
                    - token: token1
                      presets:
                        - 100
                        - 2000
                fields:
                    - binding: key1
                      name: Binding test
                      presets:
                        - value: value2
                select-tokens:
                    - key: token2
                      name: Test token
                      description: Test description
    charts:
        chart1:
            scenario: scenario1.scenario2
            plots:
                plot1:
                    title: Test title
                    subtitle: Test subtitle
                    marks:
                        - type: dot
                          options:
                            x: 1
                            y: 2
                            r: 3
                            fill: red
                            stroke: blue
                            transform:
                                type: hexbin
                                content:
                                    outputs:
                                        x: 1
                                        y: 2
                                        r: 3
                                        z: 4
                                        stroke: green
                                        fill: blue
                                    options:
                                        x: 1
                                        y: 2
                                        bin-width: 10
                        - type: line
                          options:
                            transform:
                                type: binx
                                content:
                                    outputs:
                                        x: 1
                                    options:
                                        thresholds: 10
                        - type: recty
                          options:
                            x0: 1
                            x1: 2
                            y0: 3
                            y1: 4
                    x:
                       label: Test x label
                       anchor: start
                       label-anchor: start
                       label-arrow: none
                    y:
                       label: Test y label
                       anchor: start
                       label-anchor: start
                       label-arrow: none
                    margin: 10
                    margin-left: 20
                    margin-right: 30
                    margin-top: 40
                    margin-bottom: 50
                    inset: 60
    "#;

    #[test]
    fn test_try_from_settings() {
        let config =
            Config::try_from_settings(vec![ORDERBOOK_YAML.to_string(), DOTRAIN_YAML.to_string()])
                .unwrap();
    }
}
