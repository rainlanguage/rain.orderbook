use super::config_source::ConfigSourceError;
use crate::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};
use strict_yaml_rust::StrictYaml;
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
    pub networks: HashMap<String, Arc<NetworkCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, SubgraphCfg>")
    )]
    pub subgraphs: HashMap<String, Arc<SubgraphCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, string>")
    )]
    pub metaboards: HashMap<String, Arc<Url>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, OrderbookCfg>")
    )]
    pub orderbooks: HashMap<String, Arc<OrderbookCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, TokenCfg>")
    )]
    pub tokens: HashMap<String, Arc<TokenCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, DeployerCfg>")
    )]
    pub deployers: HashMap<String, Arc<DeployerCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, OrderCfg>")
    )]
    pub orders: HashMap<String, Arc<OrderCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ScenarioCfg>")
    )]
    pub scenarios: HashMap<String, Arc<ScenarioCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ChartCfg>")
    )]
    pub charts: HashMap<String, Arc<ChartCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, DeploymentCfg>")
    )]
    pub deployments: HashMap<String, Arc<DeploymentCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub sentry: Option<bool>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub version: String,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_opt_hashmap_as_object"),
        tsify(type = "Record<string, string>", optional)
    )]
    pub accounts: Option<HashMap<String, Arc<String>>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub gui: Option<GuiCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(Config);

#[derive(Error, Debug)]
pub enum ParseConfigSourceError {
    #[error(transparent)]
    ParseNetworkConfigSourceError(#[from] ParseNetworkConfigSourceError),
    #[error(transparent)]
    ParseOrderbookConfigSourceError(#[from] ParseOrderbookConfigSourceError),
    #[error(transparent)]
    ParseTokenConfigSourceError(#[from] ParseTokenConfigSourceError),
    #[error(transparent)]
    ParseOrderConfigSourceError(#[from] ParseOrderConfigSourceError),
    #[error(transparent)]
    ParseDeployerConfigSourceError(#[from] ParseDeployerConfigSourceError),
    #[error(transparent)]
    ParseScenarioConfigSourceError(#[from] ParseScenarioConfigSourceError),
    #[error(transparent)]
    ParseChartConfigSourceError(#[from] ParseChartConfigSourceError),
    #[error(transparent)]
    ParseDeploymentConfigSourceError(#[from] ParseDeploymentConfigSourceError),
    #[error(transparent)]
    ParseGuiConfigSourceError(#[from] ParseGuiConfigSourceError),
    #[error("Failed to parse subgraph {}", 0)]
    SubgraphParseError(url::ParseError),
    #[error(transparent)]
    YamlDeserializerError(#[from] serde_yaml::Error),
    #[error(transparent)]
    ConfigSourceError(#[from] ConfigSourceError),
}

impl TryFrom<ConfigSource> for Config {
    type Error = ParseConfigSourceError;

    fn try_from(item: ConfigSource) -> Result<Self, Self::Error> {
        let networks = item
            .networks
            .into_iter()
            .map(|(name, network)| (name.clone(), Arc::new(network.into_network(name.clone()))))
            .collect::<HashMap<String, Arc<NetworkCfg>>>();

        let subgraphs = item
            .subgraphs
            .into_iter()
            .map(|(name, subgraph)| {
                (
                    name.clone(),
                    Arc::new(SubgraphCfg {
                        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
                        key: name.clone(),
                        url: subgraph.clone(),
                    }),
                )
            })
            .collect::<HashMap<String, Arc<SubgraphCfg>>>();

        let metaboards = item
            .metaboards
            .into_iter()
            .map(|(name, metaboard)| (name, Arc::new(metaboard)))
            .collect::<HashMap<String, Arc<Url>>>();

        let orderbooks = item
            .orderbooks
            .into_iter()
            .map(|(name, orderbook)| {
                Ok((
                    name.clone(),
                    Arc::new(orderbook.try_into_orderbook(name, &networks, &subgraphs)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<OrderbookCfg>>, ParseConfigSourceError>>()?;

        let tokens = item
            .tokens
            .into_iter()
            .map(|(name, token)| {
                Ok((
                    name.clone(),
                    Arc::new(token.try_into_token(&name, &networks)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<TokenCfg>>, ParseConfigSourceError>>()?;

        let deployers = item
            .deployers
            .into_iter()
            .map(|(name, deployer)| {
                Ok((
                    name.clone(),
                    Arc::new(deployer.try_into_deployer(name, &networks)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<DeployerCfg>>, ParseConfigSourceError>>()?;

        let orders = item
            .orders
            .into_iter()
            .map(|(name, order)| {
                Ok((
                    name,
                    Arc::new(order.try_into_order(&deployers, &orderbooks, &tokens)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<OrderCfg>>, ParseConfigSourceError>>()?;

        // Initialize an empty HashMap for all scenarios
        let mut scenarios = HashMap::new();

        // Directly iterate over scenarios if it's a HashMap
        for (name, scenario_string) in item.scenarios {
            let scenario_map = scenario_string.try_into_scenarios(
                name.clone(),
                &ScenarioParent::default(),
                &deployers,
            )?;

            // Merge the scenarios
            scenarios.extend(scenario_map);
        }

        let deployments = item
            .deployments
            .into_iter()
            .map(|(name, deployment)| {
                Ok((
                    name,
                    Arc::new(deployment.try_into_deployment(&scenarios, &orders)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<DeploymentCfg>>, ParseConfigSourceError>>()?;

        let charts = item
            .charts
            .into_iter()
            .map(|(name, chart)| {
                Ok((
                    name.clone(),
                    Arc::new(chart.try_into_chart(name, &scenarios)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<ChartCfg>>, ParseConfigSourceError>>()?;

        let accounts = item.accounts.map(|wl| {
            wl.into_iter()
                .map(|(name, address)| (name, Arc::new(address)))
                .collect::<HashMap<String, Arc<String>>>()
        });

        let gui = match item.gui {
            Some(g) => Some(g.try_into_gui(&deployments, &tokens)?),
            None => None,
        };

        let config = Config {
            version: item.version,
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
            sentry: item.sentry,
            accounts,
            gui,
        };

        Ok(config)
    }
}

impl Config {
    pub async fn try_from_string(val: String) -> Result<Config, ParseConfigSourceError> {
        let config_source = ConfigSource::try_from_string(val, None).await?.0;
        std::convert::TryInto::<Config>::try_into(config_source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::collections::HashMap;
    use url::Url;

    #[test]
    fn test_basic_conversion() {
        let mut networks = HashMap::new();
        networks.insert(
            "mainnet".to_string(),
            NetworkConfigSource {
                rpcs: vec![Url::parse("https://mainnet.node").unwrap()],
                chain_id: 1,
                label: Some("Ethereum Mainnet".to_string()),
                network_id: Some(1),
                currency: Some("ETH".to_string()),
            },
        );

        let mut subgraphs = HashMap::new();
        subgraphs.insert(
            "mainnet".to_string(),
            Url::parse("https://mainnet.subgraph").unwrap(),
        );

        let mut metaboards = HashMap::new();
        metaboards.insert(
            "mainnet".to_string(),
            Url::parse("https://mainnet.metaboard").unwrap(),
        );

        let mut orderbooks = HashMap::new();
        orderbooks.insert(
            "mainnetOrderbook".to_string(),
            OrderbookConfigSource {
                address: "0x1234567890123456789012345678901234567890"
                    .parse::<Address>()
                    .unwrap(),
                network: Some("mainnet".to_string()),
                subgraph: Some("mainnet".to_string()),
                label: Some("Mainnet Orderbook".to_string()),
            },
        );

        let mut tokens = HashMap::new();
        tokens.insert(
            "ETH".to_string(),
            TokenConfigSource {
                network: "mainnet".to_string(),
                address: "0x7890123456789012345678901234567890123456"
                    .parse::<Address>()
                    .unwrap(),
                decimals: Some(18),
                label: Some("Ethereum".to_string()),
                symbol: Some("ETH".to_string()),
            },
        );

        let mut deployers = HashMap::new();
        deployers.insert(
            "mainDeployer".to_string(),
            DeployerConfigSource {
                address: "0xabcdef0123456789ABCDEF0123456789ABCDEF01"
                    .parse::<Address>()
                    .unwrap(),
                network: Some("mainnet".to_string()),
                label: Some("Mainnet Deployer".to_string()),
            },
        );

        let using_networks_from = HashMap::new();
        let orders = HashMap::new();
        let scenarios = HashMap::new();
        let charts = HashMap::new();
        let deployments = HashMap::new();
        let sentry = Some(true);
        let accounts = Some(HashMap::from([(
            "name-one".to_string(),
            "address-one".to_string(),
        )]));
        let gui = Some(GuiConfigSourceCfg {
            name: "Some name".to_string(),
            description: "Some description".to_string(),
            deployments: HashMap::new(),
        });

        let config_string = ConfigSource {
            version: "1".to_string(),
            using_networks_from,
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
            accounts,
            gui,
        };

        let config_result = Config::try_from(config_string);
        assert!(config_result.is_ok());

        let config = config_result.unwrap();

        // Verify networks
        assert_eq!(config.networks.len(), 1);
        let mainnet_network = config.networks.get("mainnet").unwrap();
        assert_eq!(
            mainnet_network.rpcs,
            vec![Url::parse("https://mainnet.node").unwrap()]
        );
        assert_eq!(mainnet_network.chain_id, 1);
        assert_eq!(mainnet_network.key, "mainnet".to_string());

        // Verify subgraphs
        assert_eq!(config.subgraphs.len(), 1);
        let mainnet_subgraph = config.subgraphs.get("mainnet").unwrap();
        assert_eq!(mainnet_subgraph.url.as_str(), "https://mainnet.subgraph/");

        // Verify orderbooks
        assert_eq!(config.orderbooks.len(), 1);
        let mainnet_orderbook = config.orderbooks.get("mainnetOrderbook").unwrap();
        assert_eq!(
            mainnet_orderbook.address,
            "0x1234567890123456789012345678901234567890"
                .parse::<Address>()
                .unwrap()
        );

        // Verify tokens
        assert_eq!(config.tokens.len(), 1);
        let eth_token = config.tokens.get("ETH").unwrap();
        assert_eq!(
            eth_token.address,
            "0x7890123456789012345678901234567890123456"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(eth_token.decimals, Some(18));

        // Verify deployers
        assert_eq!(config.deployers.len(), 1);
        let main_deployer = config.deployers.get("mainDeployer").unwrap();
        assert_eq!(
            main_deployer.address,
            "0xabcdef0123456789ABCDEF0123456789ABCDEF01"
                .parse::<Address>()
                .unwrap()
        );

        // Verify sentry
        assert!(config.sentry.unwrap());

        // Verify spec_version
        assert_eq!(config.version, "1".to_string());

        // Verify accounts
        assert!(config.accounts.is_some());
        let accounts = config.accounts.as_ref().unwrap();
        assert_eq!(accounts.len(), 1);
        let (name, address) = accounts.iter().next().unwrap();
        assert_eq!(name, "name-one");
        assert_eq!(address.as_str(), "address-one");

        // Verify gui
        assert!(config.gui.is_some());
        let gui = config.gui.as_ref().unwrap();
        assert_eq!(gui.name, "Some name");
        assert_eq!(gui.description, "Some description");
    }
}
