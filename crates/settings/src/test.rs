use crate::*;
use alloy::primitives::Address;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use subgraph::SubgraphCfg;

// Helper function to create a mock network
pub fn mock_network() -> Arc<NetworkCfg> {
    Arc::new(NetworkCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "local".into(),
        rpc: ("http://127.0.0.1:8545").parse().unwrap(),
        chain_id: 1,
        label: Some("Local Testnet".into()),
        network_id: Some(1),
        currency: Some("ETH".into()),
    })
}

// Helper function to create a mock deployer
pub fn mock_deployer() -> Arc<DeployerCfg> {
    Arc::new(DeployerCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "Deployer1".to_string(),
        address: Address::repeat_byte(0x03),
        network: mock_network(),
    })
}

// Helper function to create a mock orderbook
pub fn mock_orderbook() -> Arc<OrderbookCfg> {
    Arc::new(OrderbookCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "".to_string(),
        label: Some("Orderbook1".into()),
        address: Address::repeat_byte(0x04),
        subgraph: Arc::new(SubgraphCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            url: "https://subgraph.com".parse().unwrap(),
        }),
        network: mock_network(),
    })
}

// Helper function to create a mock token
pub fn mock_token(name: &str) -> Arc<TokenCfg> {
    Arc::new(TokenCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "".to_string(),
        label: Some(name.into()),
        address: Address::repeat_byte(0x05),
        symbol: Some("TKN".into()),
        decimals: Some(18),
        network: mock_network(),
    })
}

pub fn mock_plot(name: &str) -> (String, PlotCfg) {
    (
        name.to_string(),
        PlotCfg {
            title: Some("Title".to_string()),
            subtitle: Some("Subtitle".to_string()),
            inset: None,
            margin: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            margin_top: None,
            x: None,
            y: None,
            marks: vec![MarkCfg::Dot(DotOptionsCfg {
                transform: None,
                r: None,
                fill: None,
                x: Some("0.1".to_string()),
                y: Some("0.2".to_string()),
                stroke: Some("black".to_string()),
            })],
        },
    )
}

pub fn mock_subgraph() -> Arc<SubgraphCfg> {
    Arc::new(SubgraphCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "".to_string(),
        url: "https://subgraph.com".parse().unwrap(),
    })
}

pub const MOCK_ORDERBOOK_YAML: &str = r#"
version: 1
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
accounts:
    account1: 0x0000000000000000000000000000000000000001
    account2: 0x0000000000000000000000000000000000000002
"#;

pub const MOCK_DOTRAIN_YAML: &str = r#"
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
