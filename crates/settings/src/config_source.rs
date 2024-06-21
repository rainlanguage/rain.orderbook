use crate::blocks::Blocks;
use crate::remote::chains::{chainid::ChainIdError, RemoteNetworkError, RemoteNetworks};
use crate::{Metric, Plot};
use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use typeshare::typeshare;
use url::Url;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigSource {
    #[serde(default)]
    pub using_networks_from: HashMap<String, RemoteNetworksConfigSource>,
    #[serde(default)]
    pub networks: HashMap<String, NetworkConfigSource>,
    #[serde(default)]
    pub subgraphs: HashMap<String, Url>,
    #[serde(default)]
    pub orderbooks: HashMap<String, OrderbookConfigSource>,
    #[serde(default)]
    pub tokens: HashMap<String, TokenConfigSource>,
    #[serde(default)]
    pub deployers: HashMap<String, DeployerConfigSource>,
    #[serde(default)]
    pub orders: HashMap<String, OrderConfigSource>,
    #[serde(default)]
    pub scenarios: HashMap<String, ScenarioConfigSource>,
    #[serde(default)]
    pub charts: HashMap<String, ChartConfigSource>,
    #[serde(default)]
    pub deployments: HashMap<String, DeploymentConfigSource>,
    #[serde(default)]
    pub metaboards: HashMap<String, Url>,
    pub sentry: Option<bool>,
}

#[typeshare]
pub type SubgraphRef = String;

#[typeshare]
pub type ScenarioRef = String;

#[typeshare]
pub type NetworkRef = String;

#[typeshare]
pub type DeployerRef = String;

#[typeshare]
pub type OrderRef = String;

#[typeshare]
pub type OrderbookRef = String;

#[typeshare]
pub type TokenRef = String;

#[typeshare]
pub type MetaboardRef = String;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkConfigSource {
    pub rpc: Url,
    #[typeshare(typescript(type = "number"))]
    pub chain_id: u64,
    pub label: Option<String>,
    #[typeshare(typescript(type = "number"))]
    pub network_id: Option<u64>,
    pub currency: Option<String>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteNetworksConfigSource {
    pub url: String,
    pub format: String,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OrderbookConfigSource {
    pub address: Address,
    pub network: Option<NetworkRef>,
    pub subgraph: Option<SubgraphRef>,
    pub label: Option<String>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TokenConfigSource {
    pub network: NetworkRef,
    pub address: Address,
    pub decimals: Option<u8>,
    pub label: Option<String>,
    pub symbol: Option<String>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DeployerConfigSource {
    pub address: Address,
    pub network: Option<NetworkRef>,
    pub label: Option<String>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DeploymentConfigSource {
    pub scenario: ScenarioRef,
    pub order: OrderRef,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IOString {
    pub token: TokenRef,
    #[typeshare(typescript(type = "bigint"))]
    pub vault_id: Option<U256>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OrderConfigSource {
    pub inputs: Vec<IOString>,
    pub outputs: Vec<IOString>,
    pub deployer: Option<DeployerRef>,
    pub orderbook: Option<OrderbookRef>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ScenarioConfigSource {
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[typeshare(typescript(type = "number"))]
    pub runs: Option<u64>,
    pub blocks: Option<Blocks>,
    pub deployer: Option<DeployerRef>,
    pub scenarios: Option<HashMap<String, ScenarioConfigSource>>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ChartConfigSource {
    pub scenario: Option<ScenarioRef>,
    pub plots: Option<HashMap<String, Plot>>,
    pub metrics: Option<Vec<Metric>>,
}

#[derive(Error, Debug)]
pub enum ConfigSourceError {
    #[error(transparent)]
    YamlDeserializerError(#[from] serde_yaml::Error),
    #[error(transparent)]
    RemoteNetworkError(#[from] RemoteNetworkError),
    #[error("Conflicting networks, a network with key '{}' already exists", 0)]
    ConflictingNetworks(String),
    #[error(transparent)]
    ChainIdError(#[from] ChainIdError),
}

impl ConfigSource {
    pub async fn try_from_string(val: String) -> Result<ConfigSource, ConfigSourceError> {
        let mut conf: ConfigSource = serde_yaml::from_str(&val)?;
        if !conf.using_networks_from.is_empty() {
            for (_key, item) in conf.using_networks_from.iter() {
                let remote_networks =
                    RemoteNetworks::try_from_remote_network_config_source(item.clone()).await?;
                match remote_networks {
                    RemoteNetworks::ChainId(chains) => {
                        for chain in &chains {
                            if conf.networks.iter().all(|(k, _v)| *k != chain.short_name) {
                                if let Ok(v) = chain.clone().try_into() {
                                    conf.networks.insert(chain.short_name.clone(), v);
                                }
                            } else {
                                return Err(ConfigSourceError::ConflictingNetworks(
                                    chain.name.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_yaml_into_configstrings() {
        let yaml_data = r#"
using-networks-from:
    chainid:
        url: https://chainid.network/chains.json
        format: chainid

networks:
    mainnet:
        rpc: https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet:
        rpc: https://testnet.node
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH

subgraphs:
    mainnet: https://mainnet.subgraph
    testnet: https://testnet.subgraph

orderbooks:
    mainnetOrderbook:
        address: 0xabc0000000000000000000000000000000000001
        network: mainnet
        subgraph: mainnet
        label: Mainnet Orderbook
    testnetOrderbook:
        address: 0xabc0000000000000000000000000000000000002
        network: testnet
        subgraph: testnet
        label: Testnet Orderbook

tokens:
    eth:
        network: mainnet
        address: 0xabc0000000000000000000000000000000000003
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: mainnet
        address: 0xabc0000000000000000000000000000000000004
        decimals: 18
        label: Dai
        symbol: DAI

deployers:
    mainDeployer:
        address: 0xabc0000000000000000000000000000000000005
        network: mainnet
        label: Main Deployer
    testDeployer:
        address: 0xabc0000000000000000000000000000000000006
        network: testnet
        label: Test Deployer

orders:
    buyETH:
        inputs:
            - token: eth
            - token: dai
        outputs:
            - token: dai
              vault-id: 3
        deployer: mainDeployer
        orderbook: mainnetOrderbook

scenarios:
    mainScenario:
        bindings:
            key1: value1
            key2: value2
        runs: 100
        network: mainnet
        deployer: mainDeployer
        scenarios:
            subScenario1:
                bindings:
                    key3: value3
            subScenario2:
                bindings:
                    key4: value4
charts:
    mainChart:
        scenario: mainScenario
        metrics:
        -   label: A metric
            description: A description
            unit-prefix: $
            unit-suffix: USD
            value: 0.1
        -   label: Another metric
            unit-suffix: ETH
            value: 0.2
        -   label: Yet another metric
            unit-prefix: £
            value: 0.3
        plots:
            plot1:
                title: "My plot"
                subtitle: "My subtitle"
                marks:
                -   type: dot
                    options:
                        x: "0.1"
                        y: "0.2"
                        stroke: "black"            
            plot2:
                title: "Hexbin"
                marks:
                    - type: dot
                      options:
                        transform:
                            type: hexbin
                            content:
                                outputs:
                                    fill: count
                                options:
                                    x: 0.1
                                    y: 0.2
                                    bin-width: 10
deployments:
    first-deployment:
        scenario: mainScenario
        order: sellETH
    second-deployment:
        scenario: mainScenario
        order: buyETH
        
sentry: true"#
            .to_string();

        let config = ConfigSource::try_from_string(yaml_data).await.unwrap();

        // Asserting a few values to verify successful parsing
        assert_eq!(
            config.clone().networks.get("mainnet").unwrap().rpc,
            Url::parse("https://mainnet.node").unwrap()
        );
        assert_eq!(
            config.networks.get("mainnet").unwrap().label,
            Some("Mainnet".into())
        );
        assert_eq!(
            config.subgraphs.get("mainnet"),
            Some(&Url::parse("https://mainnet.subgraph").unwrap())
        );
        assert_eq!(
            config.orderbooks.get("mainnetOrderbook").unwrap().address,
            "0xabc0000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(config.tokens.get("eth").unwrap().decimals, Some(18));
        assert!(config.sentry.unwrap());

        // remote networks fetched from remote source and converted and added to networks
        assert_eq!(
            config.clone().networks.get("eth").unwrap().rpc,
            Url::parse("https://api.mycryptoapi.com/eth").unwrap()
        );
        assert_eq!(
            config.networks.get("eth").unwrap().label,
            Some("Ethereum Mainnet".into())
        );
        assert_eq!(
            config.clone().networks.get("matic").unwrap().rpc,
            Url::parse("https://polygon-rpc.com/").unwrap()
        );
        assert_eq!(
            config.networks.get("matic").unwrap().label,
            Some("Polygon Mainnet".into())
        );

        let expected_order = OrderConfigSource {
            inputs: vec![
                IOString {
                    token: "eth".to_string(),
                    vault_id: None,
                },
                IOString {
                    token: "dai".to_string(),
                    vault_id: None,
                },
            ],
            outputs: vec![IOString {
                token: "dai".to_string(),
                vault_id: Some(U256::from(3)),
            }],
            deployer: Some("mainDeployer".to_string()),
            orderbook: Some("mainnetOrderbook".to_string()),
        };
        let order = config.orders.get("buyETH").unwrap();
        assert_eq!(order.inputs[0].token, expected_order.inputs[0].token);
        assert_eq!(order.inputs[0].vault_id, expected_order.inputs[0].vault_id);
        assert_eq!(order.inputs[1].token, expected_order.inputs[1].token);
        assert_eq!(order.inputs[1].vault_id, expected_order.inputs[1].vault_id);
        assert_eq!(order.outputs[0].token, expected_order.outputs[0].token);
        assert_eq!(
            order.outputs[0].vault_id,
            expected_order.outputs[0].vault_id
        );
        assert_eq!(order.deployer, expected_order.deployer);
        assert_eq!(order.orderbook, expected_order.orderbook);
    }
}
