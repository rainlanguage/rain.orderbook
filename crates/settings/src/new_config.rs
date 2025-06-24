use crate::accounts::AccountCfg;
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
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, serialize_hashmap_as_object};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct NewConfig {
    dotrain_order: DotrainOrderConfig,
    orderbook: OrderbookConfig,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(NewConfig);

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct OrderbookConfig {
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
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    sentry: Option<bool>,
    version: String,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, AccountCfg>", optional)
    )]
    accounts: HashMap<String, Arc<AccountCfg>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderbookConfig);

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DotrainOrderConfig {
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
        tsify(type = "Record<string, DeploymentCfg>")
    )]
    deployments: HashMap<String, Arc<DeploymentCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    gui: Option<GuiCfg>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ChartCfg>")
    )]
    charts: HashMap<String, Arc<ChartCfg>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DotrainOrderConfig);

#[derive(Error, Debug)]
pub enum ParseConfigError {
    #[error(transparent)]
    ParseNetworkCfgError(#[from] ParseNetworkConfigSourceError),
    #[error(transparent)]
    ParseOrderbookCfgError(#[from] ParseOrderbookConfigSourceError),
    #[error(transparent)]
    ParseTokenCfgError(#[from] ParseTokenConfigSourceError),
    #[error(transparent)]
    ParseOrderCfgError(#[from] ParseOrderConfigSourceError),
    #[error(transparent)]
    ParseDeployerCfgError(#[from] ParseDeployerConfigSourceError),
    #[error(transparent)]
    ParseScenarioCfgError(#[from] ParseScenarioConfigSourceError),
    #[error(transparent)]
    ParseDeploymentCfgError(#[from] ParseDeploymentConfigSourceError),
    #[error(transparent)]
    ParseGuiCfgError(#[from] ParseGuiConfigSourceError),
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
    #[error("Order not found: {0}")]
    OrderNotFound(String),
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    #[error("Deployer not found: {0}")]
    DeployerNotFound(String),
    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),
    #[error("Chart not found: {0}")]
    ChartNotFound(String),
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[cfg(target_family = "wasm")]
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] wasm_bindgen_utils::prelude::serde_wasm_bindgen::Error),
}

impl ParseConfigError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseConfigError::ParseNetworkCfgError(e) => {
                format!("Failed to parse network configuration: {}", e)
            }
            ParseConfigError::ParseOrderbookCfgError(e) => {
                format!("Failed to parse orderbook configuration: {}", e)
            }
            ParseConfigError::ParseTokenCfgError(e) => {
                format!("Failed to parse token configuration: {}", e)
            }
            ParseConfigError::ParseOrderCfgError(e) => {
                format!("Failed to parse order configuration: {}", e)
            }
            ParseConfigError::ParseDeployerCfgError(e) => {
                format!("Failed to parse deployer configuration: {}", e)
            }
            ParseConfigError::ParseScenarioCfgError(e) => {
                format!("Failed to parse scenario configuration: {}", e)
            }
            ParseConfigError::ParseDeploymentCfgError(e) => {
                format!("Failed to parse deployment configuration: {}", e)
            }
            ParseConfigError::ParseGuiCfgError(e) => {
                format!("Failed to parse GUI configuration: {}", e)
            }
            ParseConfigError::YamlError(e) => e.to_readable_msg(),
            ParseConfigError::NetworkNotFound(name) => format!(
                "The network '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::SubgraphNotFound(name) => format!(
                "The subgraph '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::MetaboardNotFound(name) => format!(
                "The metaboard '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::OrderbookNotFound(name) => format!(
                "The orderbook '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::OrderNotFound(name) => format!(
                "The order '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::TokenNotFound(name) => format!(
                "The token '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::DeployerNotFound(name) => format!(
                "The deployer '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::ScenarioNotFound(name) => format!(
                "The scenario '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::ChartNotFound(name) => format!(
                "The chart '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::DeploymentNotFound(name) => format!(
                "The deployment '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            ParseConfigError::AccountNotFound(name) => format!(
                "The account '{}' could not be found. Please check your YAML configuration.",
                name
            ),
            #[cfg(target_family = "wasm")]
            ParseConfigError::SerdeWasmBindgenError(e) => {
                format!("Data serialization error: {}", e)
            }
        }
    }
}

#[cfg(target_family = "wasm")]
impl From<ParseConfigError> for JsValue {
    fn from(value: ParseConfigError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

#[cfg(target_family = "wasm")]
impl From<ParseConfigError> for WasmEncodedError {
    fn from(value: ParseConfigError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

impl NewConfig {
    pub fn try_from_yaml(yaml: Vec<String>, validate: bool) -> Result<Self, ParseConfigError> {
        if yaml.is_empty() {
            return Err(ParseConfigError::YamlError(YamlError::EmptyFile));
        }

        let dotrain_yaml = DotrainYaml::new(yaml.clone(), validate)?;
        let orderbook_yaml = OrderbookYaml::new(yaml, validate)?;

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
        let version = orderbook_yaml.get_spec_version()?;
        let gui = dotrain_yaml.get_gui(None)?;
        let accounts = orderbook_yaml
            .get_accounts()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect::<HashMap<_, _>>();

        let orderbook_config = OrderbookConfig {
            networks,
            subgraphs,
            metaboards,
            orderbooks,
            tokens,
            deployers,
            sentry,
            version,
            accounts,
        };
        let dotrain_order_config = DotrainOrderConfig {
            orders,
            scenarios,
            deployments,
            gui,
            charts,
        };

        let config = NewConfig {
            dotrain_order: dotrain_order_config,
            orderbook: orderbook_config,
        };
        Ok(config)
    }

    pub fn get_orderbook_config(&self) -> OrderbookConfig {
        self.orderbook.clone()
    }

    pub fn get_dotrain_order_config(&self) -> DotrainOrderConfig {
        self.dotrain_order.clone()
    }

    pub fn get_networks(&self) -> &HashMap<String, Arc<NetworkCfg>> {
        &self.orderbook.networks
    }
    pub fn get_network(&self, key: &str) -> Result<&Arc<NetworkCfg>, ParseConfigError> {
        self.orderbook
            .networks
            .get(key)
            .ok_or(ParseConfigError::NetworkNotFound(key.to_string()))
    }

    pub fn get_subgraphs(&self) -> &HashMap<String, Arc<SubgraphCfg>> {
        &self.orderbook.subgraphs
    }
    pub fn get_subgraph(&self, key: &str) -> Result<&Arc<SubgraphCfg>, ParseConfigError> {
        self.orderbook
            .subgraphs
            .get(key)
            .ok_or(ParseConfigError::SubgraphNotFound(key.to_string()))
    }

    pub fn get_metaboards(&self) -> &HashMap<String, Arc<Url>> {
        &self.orderbook.metaboards
    }
    pub fn get_metaboard(&self, key: &str) -> Result<&Arc<Url>, ParseConfigError> {
        self.orderbook
            .metaboards
            .get(key)
            .ok_or(ParseConfigError::MetaboardNotFound(key.to_string()))
    }

    pub fn get_orderbooks(&self) -> &HashMap<String, Arc<OrderbookCfg>> {
        &self.orderbook.orderbooks
    }
    pub fn get_orderbook(&self, key: &str) -> Result<&Arc<OrderbookCfg>, ParseConfigError> {
        self.orderbook
            .orderbooks
            .get(key)
            .ok_or(ParseConfigError::OrderbookNotFound(key.to_string()))
    }

    pub fn get_tokens(&self) -> &HashMap<String, Arc<TokenCfg>> {
        &self.orderbook.tokens
    }
    pub fn get_token(&self, key: &str) -> Result<&Arc<TokenCfg>, ParseConfigError> {
        self.orderbook
            .tokens
            .get(key)
            .ok_or(ParseConfigError::TokenNotFound(key.to_string()))
    }

    pub fn get_deployers(&self) -> &HashMap<String, Arc<DeployerCfg>> {
        &self.orderbook.deployers
    }
    pub fn get_deployer(&self, key: &str) -> Result<&Arc<DeployerCfg>, ParseConfigError> {
        self.orderbook
            .deployers
            .get(key)
            .ok_or(ParseConfigError::DeployerNotFound(key.to_string()))
    }

    pub fn get_orders(&self) -> &HashMap<String, Arc<OrderCfg>> {
        &self.dotrain_order.orders
    }
    pub fn get_order(&self, key: &str) -> Result<&Arc<OrderCfg>, ParseConfigError> {
        self.dotrain_order
            .orders
            .get(key)
            .ok_or(ParseConfigError::OrderNotFound(key.to_string()))
    }

    pub fn get_scenarios(&self) -> &HashMap<String, Arc<ScenarioCfg>> {
        &self.dotrain_order.scenarios
    }
    pub fn get_scenario(&self, key: &str) -> Result<&Arc<ScenarioCfg>, ParseConfigError> {
        self.dotrain_order
            .scenarios
            .get(key)
            .ok_or(ParseConfigError::ScenarioNotFound(key.to_string()))
    }

    pub fn get_deployments(&self) -> &HashMap<String, Arc<DeploymentCfg>> {
        &self.dotrain_order.deployments
    }
    pub fn get_deployment(&self, key: &str) -> Result<&Arc<DeploymentCfg>, ParseConfigError> {
        self.dotrain_order
            .deployments
            .get(key)
            .ok_or(ParseConfigError::DeploymentNotFound(key.to_string()))
    }

    pub fn get_charts(&self) -> &HashMap<String, Arc<ChartCfg>> {
        &self.dotrain_order.charts
    }
    pub fn get_chart(&self, key: &str) -> Result<&Arc<ChartCfg>, ParseConfigError> {
        self.dotrain_order
            .charts
            .get(key)
            .ok_or(ParseConfigError::ChartNotFound(key.to_string()))
    }

    pub fn get_gui(&self) -> &Option<GuiCfg> {
        &self.dotrain_order.gui
    }

    pub fn get_sentry(&self) -> &Option<bool> {
        &self.orderbook.sentry
    }

    pub fn get_version(&self) -> &String {
        &self.orderbook.version
    }

    pub fn get_accounts(&self) -> &HashMap<String, Arc<AccountCfg>> {
        &self.orderbook.accounts
    }
    pub fn get_account(&self, key: &str) -> Result<&Arc<AccountCfg>, ParseConfigError> {
        self.orderbook
            .accounts
            .get(key)
            .ok_or(ParseConfigError::AccountNotFound(key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::NewConfig;
    use crate::{
        new_config::ParseConfigError,
        spec_version::SpecVersion,
        test::{MOCK_DOTRAIN_YAML, MOCK_ORDERBOOK_YAML},
        yaml::YamlError,
        BinXTransformCfg, DotOptionsCfg, HexBinTransformCfg, LineOptionsCfg, MarkCfg,
        RectYOptionsCfg, TransformCfg,
    };
    use alloy::primitives::{Address, U256};
    use std::str::FromStr;
    use url::Url;

    fn setup_config() -> NewConfig {
        NewConfig::try_from_yaml(
            vec![
                MOCK_ORDERBOOK_YAML.to_string(),
                MOCK_DOTRAIN_YAML.to_string(),
            ],
            false,
        )
        .unwrap()
    }

    #[test]
    fn test_orderbook_config() {
        let config = setup_config();
        let orderbook_config = config.get_orderbook_config();
        assert_eq!(orderbook_config.version, SpecVersion::current());
        assert_eq!(orderbook_config.networks.len(), 2);
        assert_eq!(orderbook_config.subgraphs.len(), 2);
        assert_eq!(orderbook_config.metaboards.len(), 2);
        assert_eq!(orderbook_config.orderbooks.len(), 2);
        assert_eq!(orderbook_config.tokens.len(), 2);
        assert_eq!(orderbook_config.deployers.len(), 2);
        assert_eq!(orderbook_config.accounts.len(), 2);
        assert!(orderbook_config.sentry.is_some());
    }

    #[test]
    fn test_dotrain_order_config() {
        let config = setup_config();
        let dotrain_order_config = config.get_dotrain_order_config();
        assert_eq!(dotrain_order_config.orders.len(), 1);
        assert_eq!(dotrain_order_config.scenarios.len(), 2);
        assert_eq!(dotrain_order_config.deployments.len(), 2);
        assert!(dotrain_order_config.gui.is_some());
        assert_eq!(dotrain_order_config.charts.len(), 1);
    }

    #[test]
    fn test_networks() {
        let config = setup_config();
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

        // Test missing network reference
        let err = config.get_network("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::NetworkNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_subgraphs() {
        let config = setup_config();
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

        let err = config.get_subgraph("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::SubgraphNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_metaboards() {
        let config = setup_config();
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

        let err = config.get_metaboard("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::MetaboardNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_orderbooks() {
        let config = setup_config();
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

        let err = config.get_orderbook("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::OrderbookNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_tokens() {
        let config = setup_config();
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

        let err = config.get_token("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::TokenNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_deployers() {
        let config = setup_config();
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

        let err = config.get_deployer("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::DeployerNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_orders() {
        let config = setup_config();
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

        let err = config.get_order("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::OrderNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_scenarios() {
        let config = setup_config();
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

        let err = config.get_scenario("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::ScenarioNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_deployments() {
        let config = setup_config();
        let deployment1 = config.get_deployment("deployment1").unwrap();
        assert_eq!(deployment1.key, "deployment1");
        assert_eq!(deployment1.order.key, "order1");
        assert_eq!(deployment1.scenario.key, "scenario1.scenario2");
        let deployment2 = config.get_deployment("deployment2").unwrap();
        assert_eq!(deployment2.key, "deployment2");
        assert_eq!(deployment2.order.key, "order1");
        assert_eq!(deployment2.scenario.key, "scenario1");

        let err = config.get_deployment("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::DeploymentNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_charts() {
        let config = setup_config();
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

        let err = config.get_chart("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::ChartNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_gui() {
        let config = setup_config();
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
    }

    #[test]
    fn test_sentry_and_version() {
        let config = setup_config();
        assert!(config.get_sentry().unwrap());
        assert_eq!(config.get_version(), &SpecVersion::current());
    }

    #[test]
    fn test_accounts() {
        let config = setup_config();
        let account1 = config.get_account("account1").unwrap();
        assert_eq!(account1.key, "account1");
        assert_eq!(
            account1.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        let account2 = config.get_account("account2").unwrap();
        assert_eq!(account2.key, "account2");
        assert_eq!(
            account2.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );

        let err = config.get_account("nonexistent").unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::AccountNotFound(ref s) if s == "nonexistent"
        ));
    }

    #[test]
    fn test_yaml_parsing_errors() {
        let invalid_yaml_syntax = vec!["invalid: yaml: [unclosed".to_string()];
        let result = NewConfig::try_from_yaml(invalid_yaml_syntax, false);
        assert!(matches!(result, Err(ParseConfigError::YamlError(_))));

        let malformed_yaml_structure = vec!["key: value: invalid".to_string()];
        let result = NewConfig::try_from_yaml(malformed_yaml_structure, false);
        assert!(matches!(result, Err(ParseConfigError::YamlError(_))));

        let unclosed_brackets = vec!["items: [item1, item2".to_string()];
        let result = NewConfig::try_from_yaml(unclosed_brackets, false);
        assert!(matches!(result, Err(ParseConfigError::YamlError(_))));

        let invalid_indentation = vec![r#"
key1: value1
  key2: value2
 key3: value3
"#
        .to_string()];
        let result = NewConfig::try_from_yaml(invalid_indentation, false);
        assert!(matches!(result, Err(ParseConfigError::YamlError(_))));

        let empty_string = vec!["".to_string()];
        let err = NewConfig::try_from_yaml(empty_string, false).unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::YamlError(YamlError::EmptyFile)
        ));

        let empty_yaml = vec![];
        let err = NewConfig::try_from_yaml(empty_yaml, false).unwrap_err();
        assert!(matches!(
            err,
            ParseConfigError::YamlError(YamlError::EmptyFile)
        ));
    }
}
