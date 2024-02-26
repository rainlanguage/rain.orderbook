use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigString {
    pub networks: Option<HashMap<String, NetworkString>>,
    pub subgraphs: Option<HashMap<String, String>>,
    pub orderbooks: Option<HashMap<String, OrderbookString>>,
    pub vaults: Option<HashMap<String, String>>,
    pub tokens: Option<HashMap<String, TokenString>>,
    pub deployers: Option<HashMap<String, DeployerString>>,
    pub orders: Option<HashMap<String, OrderString>>,
    pub scenarios: Option<HashMap<String, ScenarioString>>,
    pub charts: Option<HashMap<String, ChartString>>,
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
pub struct OrderString {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub network: String,
    pub deployer: Option<String>,
    pub orderbook: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScenarioString {
    pub bindings: HashMap<String, String>,
    pub runs: Option<String>,
    pub network: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

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
              "#;

        let config: ConfigString = serde_yaml::from_str(yaml_data).unwrap();

        // Asserting a few values to verify successful parsing
        assert_eq!(
            config.clone().networks.unwrap().get("mainnet").unwrap().rpc,
            "https://mainnet.node".to_string()
        );
        assert_eq!(
            config.networks.unwrap().get("mainnet").unwrap().label,
            Some("Mainnet".into())
        );
        assert_eq!(
            config.subgraphs.unwrap().get("mainnet"),
            Some(&"https://mainnet.subgraph".to_string())
        );
        assert_eq!(
            config
                .orderbooks
                .unwrap()
                .get("mainnetOrderbook")
                .unwrap()
                .address,
            "0x123".to_string()
        );
        assert_eq!(
            config.tokens.unwrap().get("eth").unwrap().decimals,
            Some("18".to_string())
        );
    }
}
