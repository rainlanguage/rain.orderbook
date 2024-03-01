use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConfigString {
    #[serde(default)]
    pub networks: HashMap<String, NetworkString>,
    #[serde(default)]
    pub subgraphs: HashMap<String, String>,
    #[serde(default)]
    pub orderbooks: HashMap<String, OrderbookString>,
    #[serde(default)]
    pub vaults: HashMap<String, String>,
    #[serde(default)]
    pub tokens: HashMap<String, TokenString>,
    #[serde(default)]
    pub deployers: HashMap<String, DeployerString>,
    #[serde(default)]
    pub orders: HashMap<String, OrderString>,
    #[serde(default)]
    pub scenarios: HashMap<String, ScenarioString>,
    #[serde(default)]
    pub charts: HashMap<String, ChartString>,
    #[serde(default)]
    pub deployments: HashMap<String, DeploymentString>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkString {
    pub rpc: String,
    pub chain_id: String,
    pub label: Option<String>,
    pub network_id: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderbookString {
    pub address: String,
    pub network: Option<String>,
    pub subgraph: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenString {
    pub network: String,
    pub address: String,
    pub decimals: Option<String>,
    pub label: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployerString {
    pub address: String,
    pub network: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentString {
    pub scenario: String,
    pub order: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderString {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub network: String,
    pub deployer: Option<String>,
    pub orderbook: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScenarioString {
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    pub runs: Option<String>,
    pub deployer: Option<String>,
    pub orderbook: Option<String>,
    pub scenarios: Option<HashMap<String, ScenarioString>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartString {
    pub scenario: Option<String>,
    pub plots: HashMap<String, PlotString>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotString {
    pub data: DataPointsString,
    pub plot_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataPointsString {
    pub x: String,
    pub y: String,
}

impl TryFrom<String> for ConfigString {
    type Error = serde_yaml::Error;
    fn try_from(val: String) -> Result<ConfigString, Self::Error> {
        serde_yaml::from_str(&val)
    }
}

impl TryFrom<&str> for ConfigString {
    type Error = serde_yaml::Error;
    fn try_from(val: &str) -> Result<ConfigString, Self::Error> {
        serde_yaml::from_str(val)
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
        chain_id: 1
        label: Mainnet
        network_id: 1
        currency: ETH
    testnet:
        rpc: https://testnet.node
        chain_id: 2
        label: Testnet
        network_id: 2
        currency: ETH

subgraphs:
    mainnet: https://mainnet.subgraph
    testnet: https://testnet.subgraph

orderbooks:
    mainnetOrderbook:
        address: 0x123
        network: mainnet
        subgraph: mainnet
        label: Mainnet Orderbook
    testnetOrderbook:
        address: 0x456
        network: testnet
        subgraph: testnet
        label: Testnet Orderbook

vaults:
    mainVault: 0x789
    testVault: 0xabc

tokens:
    eth:
        network: mainnet
        address: 0xdef
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: mainnet
        address: 0xghi
        decimals: 18
        label: Dai
        symbol: DAI

deployers:
    mainDeployer:
        address: 0xjkl
        network: mainnet
        label: Main Deployer
    testDeployer:
        address: 0xmnop
        network: testnet
        label: Test Deployer

orders:
    buyETH:
        inputs:
        - eth
        outputs:
        - dai
        network: mainnet
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
        orderbook: mainnetOrderbook
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
                plot_type: line
            plot2:
                data:
                    x: dataX2
                    y: dataY2
                plot_type: bar
deployments:
    first-deployment:
        scenario: mainScenario
        order: bytETH
    second-deployment:
        scenario: mainScenario
        order: buyETH"#;

        let config: ConfigString = yaml_data.try_into().unwrap();

        // Asserting a few values to verify successful parsing
        assert_eq!(
            config.clone().networks.get("mainnet").unwrap().rpc,
            "https://mainnet.node".to_string()
        );
        assert_eq!(
            config.networks.get("mainnet").unwrap().label,
            Some("Mainnet".into())
        );
        assert_eq!(
            config.subgraphs.get("mainnet"),
            Some(&"https://mainnet.subgraph".to_string())
        );
        assert_eq!(
            config.orderbooks.get("mainnetOrderbook").unwrap().address,
            "0x123".to_string()
        );
        assert_eq!(
            config.tokens.get("eth").unwrap().decimals,
            Some("18".to_string())
        );
    }
}
