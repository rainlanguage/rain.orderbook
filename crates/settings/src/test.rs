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
