use crate::*;
use alloy_primitives::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use typeshare::typeshare;
use url::Url;

#[typeshare]
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[typeshare(typescript(type = "Record<string, Network>"))]
    pub networks: HashMap<String, Arc<Network>>,
    #[typeshare(typescript(type = "Record<string, string>"))]
    pub subgraphs: HashMap<String, Arc<Subgraph>>,
    #[typeshare(typescript(type = "Record<string, string>"))]
    pub vaults: HashMap<String, Arc<Vault>>,
    #[typeshare(typescript(type = "Record<string, Orderbook>"))]
    pub orderbooks: HashMap<String, Arc<Orderbook>>,
    #[typeshare(typescript(type = "Record<string, Token>"))]
    pub tokens: HashMap<String, Arc<Token>>,
    #[typeshare(typescript(type = "Record<string, Deployer>"))]
    pub deployers: HashMap<String, Arc<Deployer>>,
    #[typeshare(typescript(type = "Record<string, Order>"))]
    pub orders: HashMap<String, Arc<Order>>,
    #[typeshare(typescript(type = "Record<string, Scenario>"))]
    pub scenarios: HashMap<String, Arc<Scenario>>,
    #[typeshare(typescript(type = "Record<string, Chart>"))]
    pub charts: HashMap<String, Arc<Chart>>,
}

pub type Subgraph = Url;
pub type Vault = U256;

#[derive(Error, Debug)]
pub enum ParseConfigStringError {
    #[error(transparent)]
    ParseNetworkStringError(#[from] ParseNetworkStringError),
    #[error(transparent)]
    ParseOrderbookStringError(#[from] ParseOrderbookStringError),
    #[error(transparent)]
    ParseTokenStringError(#[from] ParseTokenStringError),
    #[error(transparent)]
    ParseOrderStringError(#[from] ParseOrderStringError),
    #[error(transparent)]
    ParseDeployerStringError(#[from] ParseDeployerStringError),
    #[error(transparent)]
    ParseScenarioStringError(#[from] ParseScenarioStringError),
    #[error(transparent)]
    ParseChartStringError(#[from] ParseChartStringError),
    #[error("Failed to parse vault {}", 0)]
    VaultParseError(alloy_primitives::ruint::ParseError),
    #[error("Failed to parse subgraph {}", 0)]
    SubgraphParseError(url::ParseError),
    #[error(transparent)]
    YamlDeserializerError(#[from] serde_yaml::Error),
}

impl TryFrom<ConfigString> for Config {
    type Error = ParseConfigStringError;

    fn try_from(item: ConfigString) -> Result<Self, Self::Error> {
        let networks = item
            .networks
            .into_iter()
            .map(|(name, network)| Ok((name, Arc::new(network.try_into()?))))
            .collect::<Result<HashMap<String, Arc<Network>>, ParseConfigStringError>>()?;

        let subgraphs = item
            .subgraphs
            .into_iter()
            .map(|(name, subgraph)| {
                Ok((
                    name,
                    Arc::new(
                        subgraph
                            .parse()
                            .map_err(ParseConfigStringError::SubgraphParseError)?,
                    ),
                ))
            })
            .collect::<Result<HashMap<String, Arc<Subgraph>>, ParseConfigStringError>>()?;

        let vaults = item
            .vaults
            .into_iter()
            .map(|(name, vault)| {
                Ok((
                    name,
                    Arc::new(
                        vault
                            .parse::<U256>()
                            .map_err(ParseConfigStringError::VaultParseError)?,
                    ),
                ))
            })
            .collect::<Result<HashMap<String, Arc<Vault>>, ParseConfigStringError>>()?;

        let orderbooks = item
            .orderbooks
            .into_iter()
            .map(|(name, orderbook)| {
                Ok((
                    name.clone(),
                    Arc::new(orderbook.try_into_orderbook(name, &networks, &subgraphs)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<Orderbook>>, ParseConfigStringError>>()?;

        let tokens = item
            .tokens
            .into_iter()
            .map(|(name, token)| Ok((name, Arc::new(token.try_into_token(&networks)?))))
            .collect::<Result<HashMap<String, Arc<Token>>, ParseConfigStringError>>()?;

        let deployers = item
            .deployers
            .into_iter()
            .map(|(name, deployer)| {
                Ok((
                    name.clone(),
                    Arc::new(deployer.try_into_deployer(name, &networks)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<Deployer>>, ParseConfigStringError>>()?;

        let orders = item
            .orders
            .into_iter()
            .map(|(name, order)| {
                Ok((
                    name,
                    Arc::new(order.try_into_order(&networks, &deployers, &orderbooks, &tokens)?),
                ))
            })
            .collect::<Result<HashMap<String, Arc<Order>>, ParseConfigStringError>>()?;

        // Initialize an empty HashMap for all scenarios
        let mut scenarios = HashMap::new();

        // Directly iterate over scenarios if it's a HashMap
        for (name, scenario_string) in item.scenarios {
            let scenario_map = scenario_string.try_into_scenarios(
                name.clone(),
                &ScenarioParent::default(),
                &deployers,
                &orderbooks,
            )?;

            // Merge the scenarios
            scenarios.extend(scenario_map);
        }

        let config = Config {
            networks,
            subgraphs,
            vaults,
            orderbooks,
            tokens,
            deployers,
            orders,
            scenarios,
            charts: HashMap::new(),
        };

        Ok(config)
    }
}

impl TryFrom<String> for Config {
    type Error = ParseConfigStringError;
    fn try_from(val: String) -> Result<Config, Self::Error> {
        std::convert::TryInto::<ConfigString>::try_into(val)?.try_into()
    }
}

impl TryFrom<&str> for Config {
    type Error = ParseConfigStringError;
    fn try_from(val: &str) -> Result<Config, Self::Error> {
        std::convert::TryInto::<ConfigString>::try_into(val)?.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;
    use std::collections::HashMap;
    use url::Url;

    #[test]
    fn test_basic_conversion() {
        let mut networks = HashMap::new();
        networks.insert(
            "mainnet".to_string(),
            NetworkString {
                rpc: "https://mainnet.node".to_string(),
                chain_id: "1".to_string(),
                label: Some("Ethereum Mainnet".to_string()),
                network_id: Some("1".to_string()),
                currency: Some("ETH".to_string()),
            },
        );

        let mut subgraphs = HashMap::new();
        subgraphs.insert(
            "mainnet".to_string(),
            "https://mainnet.subgraph".to_string(),
        );

        let mut orderbooks = HashMap::new();
        orderbooks.insert(
            "mainnetOrderbook".to_string(),
            OrderbookString {
                address: "0x1234567890123456789012345678901234567890".to_string(),
                network: Some("mainnet".to_string()),
                subgraph: Some("mainnet".to_string()),
                label: Some("Mainnet Orderbook".to_string()),
            },
        );

        let mut vaults = HashMap::new();
        vaults.insert(
            "mainVault".to_string(),
            "0x4567890123456789012345678901234567890123".to_string(),
        );

        let mut tokens = HashMap::new();
        tokens.insert(
            "ETH".to_string(),
            TokenString {
                network: "mainnet".to_string(),
                address: "0x7890123456789012345678901234567890123456".to_string(),
                decimals: Some("18".to_string()),
                label: Some("Ethereum".to_string()),
                symbol: Some("ETH".to_string()),
            },
        );

        let mut deployers = HashMap::new();
        deployers.insert(
            "mainDeployer".to_string(),
            DeployerString {
                address: "0xabcdef0123456789ABCDEF0123456789ABCDEF01".to_string(),
                network: Some("mainnet".to_string()),
                label: Some("Mainnet Deployer".to_string()),
            },
        );

        let orders = HashMap::new();
        let scenarios = HashMap::new();
        let charts = HashMap::new();

        let config_string = ConfigString {
            networks,
            subgraphs,
            orderbooks,
            vaults,
            tokens,
            deployers,
            orders,
            scenarios,
            charts,
        };

        let config_result = Config::try_from(config_string);
        assert!(config_result.is_ok());

        let config = config_result.unwrap();

        // Verify networks
        assert_eq!(config.networks.len(), 1);
        let mainnet_network = config.networks.get("mainnet").unwrap();
        assert_eq!(
            mainnet_network.rpc,
            Url::parse("https://mainnet.node").unwrap()
        );
        assert_eq!(mainnet_network.chain_id, 1);

        // Verify subgraphs
        assert_eq!(config.subgraphs.len(), 1);
        let mainnet_subgraph = config.subgraphs.get("mainnet").unwrap();
        assert_eq!(mainnet_subgraph.as_str(), "https://mainnet.subgraph/");

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
    }
}
