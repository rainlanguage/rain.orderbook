use crate::*;
use alloy_primitives::Address;
use std::sync::Arc;

// Helper function to create a mock network
pub fn mock_network() -> Arc<Network> {
    Arc::new(Network {
        rpc: ("http://127.0.0.1:8545").parse().unwrap(),
        chain_id: 1,
        label: Some("Local Testnet".into()),
        network_id: Some(1),
        currency: Some("ETH".into()),
    })
}

// Helper function to create a mock deployer
pub fn mock_deployer() -> Arc<Deployer> {
    Arc::new(Deployer {
        address: Address::repeat_byte(0x03),
        network: Some(mock_network()),
        label: Some("Deployer1".into()),
    })
}

// Helper function to create a mock orderbook
pub fn mock_orderbook() -> Arc<Orderbook> {
    Arc::new(Orderbook {
        label: Some("Orderbook1".into()),
        address: Address::repeat_byte(0x04),
        subgraph: Arc::new("https://subgraph.com".parse().unwrap()),
        network: mock_network(),
    })
}

// Helper function to create a mock token
pub fn mock_token(name: &str) -> Arc<Token> {
    Arc::new(Token {
        label: Some(name.into()),
        address: Address::repeat_byte(0x05),
        symbol: Some("TKN".into()),
        decimals: Some(18),
        network: mock_network(),
    })
}

pub fn mock_subgraph() -> Arc<Subgraph> {
    Arc::new("http://subgraph.com".parse().unwrap())
}
