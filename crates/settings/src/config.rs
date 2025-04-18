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
            .get_networks()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let subgraphs = orderbook_yaml
            .get_subgraphs()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let metaboards = orderbook_yaml
            .get_metaboards()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v.url)))
            .collect::<HashMap<_, _>>();
        let orderbooks = orderbook_yaml
            .get_orderbooks()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let tokens = orderbook_yaml
            .get_tokens()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let deployers = orderbook_yaml
            .get_deployers()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let orders = dotrain_yaml
            .get_orders()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let scenarios = dotrain_yaml
            .get_scenarios()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let deployments = dotrain_yaml
            .get_deployments()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();
        let charts = dotrain_yaml
            .get_charts()
            .unwrap_or_default()
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
    use crate::{
        BinXTransformCfg, DotOptionsCfg, HexBinTransformCfg, LineOptionsCfg, MarkCfg,
        RectYOptionsCfg, TransformCfg,
    };
    use alloy::primitives::{Address, U256};
    use std::str::FromStr;
    use url::Url;

    use super::Config;

    const ORDERBOOK_YAML: &str = r#"
    raindex-version: 0.1.0
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
        testnet:
            rpc: https://testnet.infura.io
            chain-id: 1337
    subgraphs:
        mainnet: https://mainnet-subgraph.com
        testnet: https://testnet-subgraph.com
    metaboards:
        mainnet: https://mainnet-metaboard.com
        testnet: https://testnet-metaboard.com
    orderbooks:
        mainnet:
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            network: mainnet
            subgraph: mainnet
        testnet:
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            network: testnet
            subgraph: testnet
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
    sentry: true
    "#;
    const DOTRAIN_YAML: &str = r#"
    orders:
        order1:
            deployer: scenario1
            orderbook: mainnet
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

        assert_eq!(config.get_networks().len(), 2);
        assert_eq!(config.get_subgraphs().len(), 2);
        assert_eq!(config.get_metaboards().len(), 2);
        assert_eq!(config.get_orderbooks().len(), 2);
        assert_eq!(config.get_tokens().len(), 2);
        assert_eq!(config.get_deployers().len(), 2);
        assert_eq!(config.get_orders().len(), 1);
        assert_eq!(config.get_scenarios().len(), 2);
        assert_eq!(config.get_deployments().len(), 2);
        assert_eq!(config.get_charts().len(), 1);
        assert!(config.get_gui().is_some());
        assert!(config.get_sentry().is_some());
        assert!(config.get_raindex_version().is_some());
        assert!(config.get_accounts().is_none());

        let mainnet_network = config.get_network("mainnet").unwrap();
        assert_eq!(mainnet_network.key, "mainnet");
        assert_eq!(
            mainnet_network.rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );
        assert_eq!(mainnet_network.chain_id, 1);
        assert!(mainnet_network.label.is_none());
        assert!(mainnet_network.network_id.is_none());
        assert!(mainnet_network.currency.is_none());
        let testnet_network = config.get_network("testnet").unwrap();
        assert_eq!(testnet_network.key, "testnet");
        assert_eq!(
            testnet_network.rpc,
            Url::parse("https://testnet.infura.io").unwrap()
        );
        assert_eq!(testnet_network.chain_id, 1337);
        assert!(testnet_network.label.is_none());
        assert!(testnet_network.network_id.is_none());
        assert!(testnet_network.currency.is_none());

        let mainnet_subgraph = config.get_subgraph("mainnet").unwrap();
        assert_eq!(mainnet_subgraph.key, "mainnet");
        assert_eq!(
            mainnet_subgraph.url,
            Url::parse("https://mainnet-subgraph.com").unwrap()
        );
        let testnet_subgraph = config.get_subgraph("testnet").unwrap();
        assert_eq!(testnet_subgraph.key, "testnet");
        assert_eq!(
            testnet_subgraph.url,
            Url::parse("https://testnet-subgraph.com").unwrap()
        );

        let mainnet_metaboard = config.get_metaboard("mainnet").unwrap();
        assert_eq!(
            **mainnet_metaboard,
            Url::parse("https://mainnet-metaboard.com").unwrap()
        );
        let testnet_metaboard = config.get_metaboard("testnet").unwrap();
        assert_eq!(
            **testnet_metaboard,
            Url::parse("https://testnet-metaboard.com").unwrap()
        );

        let mainnet_orderbook = config.get_orderbook("mainnet").unwrap();
        assert_eq!(mainnet_orderbook.key, "mainnet");
        assert_eq!(
            mainnet_orderbook.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(mainnet_orderbook.network.key, "mainnet");
        assert_eq!(mainnet_orderbook.subgraph.key, "mainnet");
        assert!(mainnet_orderbook.label.is_none());
        let testnet_orderbook = config.get_orderbook("testnet").unwrap();
        assert_eq!(testnet_orderbook.key, "testnet");
        assert_eq!(
            testnet_orderbook.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(testnet_orderbook.network.key, "testnet");
        assert_eq!(testnet_orderbook.subgraph.key, "testnet");
        assert!(testnet_orderbook.label.is_none());

        let token1 = config.get_token("token1").unwrap();
        assert_eq!(token1.key, "token1");
        assert_eq!(token1.network.key, "mainnet");
        assert_eq!(
            token1.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(token1.decimals, Some(18));
        assert_eq!(token1.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token1.symbol, Some("WETH".to_string()));
        let token2 = config.get_token("token2").unwrap();
        assert_eq!(token2.key, "token2");
        assert_eq!(token2.network.key, "mainnet");
        assert_eq!(
            token2.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(token2.decimals, Some(6));
        assert_eq!(token2.label, Some("USD Coin".to_string()));
        assert_eq!(token2.symbol, Some("USDC".to_string()));

        let deployer_scenario1 = config.get_deployer("scenario1").unwrap();
        assert_eq!(deployer_scenario1.key, "scenario1");
        assert_eq!(
            deployer_scenario1.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(deployer_scenario1.network.key, "mainnet");
        let deployer2 = config.get_deployer("deployer2").unwrap();
        assert_eq!(deployer2.key, "deployer2");
        assert_eq!(
            deployer2.address,
            Address::from_str("0x0000000000000000000000000000000000000003").unwrap()
        );
        assert_eq!(deployer2.network.key, "testnet");

        let order1 = config.get_order("order1").unwrap();
        assert_eq!(order1.key, "order1");
        assert_eq!(order1.network.key, "mainnet");
        assert_eq!(order1.deployer.clone().unwrap().key, "scenario1");
        assert_eq!(order1.orderbook.clone().unwrap().key, "mainnet");
        assert_eq!(order1.inputs.len(), 1);
        let order1_input = &order1.inputs[0];
        assert_eq!(order1_input.token.as_ref().unwrap().key, "token1");
        assert_eq!(order1_input.vault_id, Some(U256::from(1)));
        assert_eq!(order1.outputs.len(), 1);
        let order1_output = &order1.outputs[0];
        assert_eq!(order1_output.token.as_ref().unwrap().key, "token2");
        assert_eq!(order1_output.vault_id, Some(U256::from(2)));

        let scenario1 = config.get_scenario("scenario1").unwrap();
        assert_eq!(scenario1.key, "scenario1");
        assert_eq!(scenario1.bindings.len(), 1);
        assert_eq!(scenario1.bindings.get("key1").unwrap(), "value1");
        assert!(scenario1.runs.is_none());
        assert!(scenario1.blocks.is_none());
        assert_eq!(scenario1.deployer.key, "scenario1");
        let scenario1_scenario2 = config.get_scenario("scenario1.scenario2").unwrap();
        assert_eq!(scenario1_scenario2.key, "scenario1.scenario2");
        assert_eq!(scenario1_scenario2.bindings.len(), 2);
        assert_eq!(scenario1_scenario2.bindings.get("key1").unwrap(), "value1");
        assert_eq!(scenario1_scenario2.bindings.get("key2").unwrap(), "value2");
        assert_eq!(scenario1_scenario2.runs.unwrap(), 10);
        assert!(scenario1_scenario2.blocks.is_none());
        assert_eq!(scenario1_scenario2.deployer.key, "scenario1");

        let deployment1 = config.get_deployment("deployment1").unwrap();
        assert_eq!(deployment1.key, "deployment1");
        assert_eq!(deployment1.order.key, "order1");
        assert_eq!(deployment1.scenario.key, "scenario1.scenario2");
        let deployment2 = config.get_deployment("deployment2").unwrap();
        assert_eq!(deployment2.key, "deployment2");
        assert_eq!(deployment2.order.key, "order1");
        assert_eq!(deployment2.scenario.key, "scenario1");

        let chart1 = config.get_chart("chart1").unwrap();
        assert_eq!(chart1.key, "chart1");
        assert_eq!(chart1.scenario.key, "scenario1.scenario2");
        assert!(chart1.plots.is_some());
        assert!(chart1.metrics.is_none());
        let plots = chart1.plots.as_ref().unwrap();
        assert_eq!(plots.len(), 1);
        let plot1 = &plots[0];
        assert_eq!(plot1.title, Some("Test title".to_string()));
        assert_eq!(plot1.subtitle, Some("Test subtitle".to_string()));
        assert_eq!(plot1.marks.len(), 3);
        match &plot1.marks[0] {
            MarkCfg::Dot(DotOptionsCfg {
                x,
                y,
                r,
                fill,
                stroke,
                transform,
            }) => {
                assert_eq!(x.as_deref(), Some("1"));
                assert_eq!(y.as_deref(), Some("2"));
                assert_eq!(r, &Some(3));
                assert_eq!(fill.as_deref(), Some("red"));
                assert_eq!(stroke.as_deref(), Some("blue"));
                assert!(transform.is_some());
                match transform.as_ref().unwrap() {
                    TransformCfg::HexBin(HexBinTransformCfg { outputs, options }) => {
                        assert_eq!(outputs.x.as_deref(), Some("1"));
                        assert_eq!(outputs.y.as_deref(), Some("2"));
                        assert_eq!(outputs.r, Some(3));
                        assert_eq!(outputs.z.as_deref(), Some("4"));
                        assert_eq!(outputs.stroke.as_deref(), Some("green"));
                        assert_eq!(outputs.fill.as_deref(), Some("blue"));
                        assert_eq!(options.x.as_deref(), Some("1"));
                        assert_eq!(options.y.as_deref(), Some("2"));
                        assert_eq!(options.bin_width, Some(10));
                    }
                    _ => panic!("Incorrect transform type for mark 0"),
                }
            }
            _ => panic!("Incorrect mark type for mark 0"),
        }
        match &plot1.marks[1] {
            MarkCfg::Line(LineOptionsCfg { transform, .. }) => {
                assert!(transform.is_some());
                match transform.as_ref().unwrap() {
                    TransformCfg::BinX(BinXTransformCfg { outputs, options }) => {
                        assert_eq!(outputs.x.as_deref(), Some("1"));
                        // other outputs not specified, should be None
                        assert!(outputs.y.is_none());
                        assert!(outputs.r.is_none());
                        assert!(outputs.z.is_none());
                        assert!(outputs.stroke.is_none());
                        assert!(outputs.fill.is_none());
                        // options x not specified, should be None
                        assert!(options.x.is_none());
                        assert_eq!(options.thresholds, Some(10));
                    }
                    _ => panic!("Incorrect transform type for mark 1"),
                }
            }
            _ => panic!("Incorrect mark type for mark 1"),
        }
        match &plot1.marks[2] {
            MarkCfg::RectY(RectYOptionsCfg {
                x0,
                x1,
                y0,
                y1,
                transform,
            }) => {
                assert_eq!(x0.as_deref(), Some("1"));
                assert_eq!(x1.as_deref(), Some("2"));
                assert_eq!(y0.as_deref(), Some("3"));
                assert_eq!(y1.as_deref(), Some("4"));
                assert!(transform.is_none());
            }
            _ => panic!("Incorrect mark type for mark 2"),
        }
        assert!(plot1.x.is_some());
        let axis_x = plot1.x.as_ref().unwrap();
        assert_eq!(axis_x.label, Some("Test x label".to_string()));
        assert_eq!(axis_x.anchor, Some("start".to_string()));
        assert_eq!(axis_x.label_anchor, Some("start".to_string()));
        assert_eq!(axis_x.label_arrow, Some("none".to_string()));
        assert!(plot1.y.is_some());
        let axis_y = plot1.y.as_ref().unwrap();
        assert_eq!(axis_y.label, Some("Test y label".to_string()));
        assert_eq!(axis_y.anchor, Some("start".to_string()));
        assert_eq!(axis_y.label_anchor, Some("start".to_string()));
        assert_eq!(axis_y.label_arrow, Some("none".to_string()));
        assert_eq!(plot1.margin, Some(10));
        assert_eq!(plot1.margin_left, Some(20));
        assert_eq!(plot1.margin_right, Some(30));
        assert_eq!(plot1.margin_top, Some(40));
        assert_eq!(plot1.margin_bottom, Some(50));
        assert_eq!(plot1.inset, Some(60));

        let gui = config.get_gui().as_ref().unwrap();
        assert_eq!(gui.name, "Test gui");
        assert_eq!(gui.description, "Test description");
        assert_eq!(gui.deployments.len(), 1);
        let gui_deployment1 = gui.deployments.get("deployment1").unwrap();
        assert_eq!(gui_deployment1.key, "deployment1");
        assert_eq!(gui_deployment1.deployment.key, "deployment1");
        assert_eq!(gui_deployment1.name, "Test deployment");
        assert_eq!(gui_deployment1.description, "Test description");
        assert_eq!(gui_deployment1.deposits.len(), 1);
        let deposit1 = &gui_deployment1.deposits[0];
        assert_eq!(deposit1.token.as_ref().unwrap().key, "token1");
        assert_eq!(
            deposit1.presets,
            Some(vec!["100".to_string(), "2000".to_string()])
        );
        assert_eq!(gui_deployment1.fields.len(), 1);
        let field1 = &gui_deployment1.fields[0];
        assert_eq!(field1.binding, "key1");
        assert_eq!(field1.name, "Binding test");
        assert!(field1.description.is_none());
        assert!(field1.presets.is_some());
        let field1_presets = field1.presets.as_ref().unwrap();
        assert_eq!(field1_presets.len(), 1);
        assert!(field1_presets[0].name.is_none());
        assert_eq!(field1_presets[0].value, "value2");
        assert!(field1.default.is_none());
        assert!(field1.show_custom_field.is_none());
        assert!(gui_deployment1.select_tokens.is_some());
        let select_tokens = gui_deployment1.select_tokens.as_ref().unwrap();
        assert_eq!(select_tokens.len(), 1);
        let select_token1 = &select_tokens[0];
        assert_eq!(select_token1.key, "token2");
        assert_eq!(select_token1.name, Some("Test token".to_string()));
        assert_eq!(
            select_token1.description,
            Some("Test description".to_string())
        );

        let sentry = config.get_sentry().unwrap();
        assert_eq!(sentry, true);
        let raindex_version = config.get_raindex_version().as_ref().unwrap();
        assert_eq!(raindex_version, "0.1.0");
    }
}
