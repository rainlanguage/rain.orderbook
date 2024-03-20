use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use typeshare::typeshare;
use url::Url;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigSource {
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
    pub vault_id: U256,
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
    pub deployer: Option<DeployerRef>,
    pub scenarios: Option<HashMap<String, ScenarioConfigSource>>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ChartConfigSource {
    pub scenario: Option<ScenarioRef>,
    pub plots: HashMap<String, PlotString>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PlotString {
    pub data: DataPointsString,
    pub plot_type: String,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DataPointsString {
    pub x: String,
    pub y: String,
}

impl TryFrom<String> for ConfigSource {
    type Error = serde_yaml::Error;
    fn try_from(val: String) -> Result<ConfigSource, Self::Error> {
        serde_yaml::from_str(&val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_yaml_into_configstrings() {
        let yaml_data = r#"
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
              vault-id: 2
            - token: dai
              vault-id: 0x1
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
        plots:
            plot1:
                data:
                    x: dataX
                    y: dataY
                plot-type: line
            plot2:
                data:
                    x: dataX2
                    y: dataY2
                plot-type: bar
deployments:
    first-deployment:
        scenario: mainScenario
        order: sellETH
    second-deployment:
        scenario: mainScenario
        order: buyETH
        
sentry: true"#
            .to_string();

        let config: ConfigSource = yaml_data.try_into().unwrap();

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
        assert_eq!(config.sentry.unwrap(), true);
    }
}
