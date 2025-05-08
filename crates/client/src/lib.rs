//! The OrderbookClient maintains a mapping of chain IDs to their respective configurations,
//! including subgraph URLs, RPC endpoints, and orderbook contract addresses.
//!
//! When methods are called without a specific chain ID, the client will query all configured networks.
//! For methods requiring a specific network (e.g., get_order), the client will:
//! 1. Use the provided chain ID if specified
//! 2. Attempt to determine the appropriate network from the order hash or other parameters
//! 3. Fall back to querying all networks if necessary

use alloy::primitives::Address;
use rain_orderbook_app_settings::yaml::orderbook::OrderbookYaml;
use std::collections::BTreeMap;

struct ChainSpecificConfig {
    subgraph_url: String,
    rpc_url: String,
    orderbook_address: Address,
}

pub struct OrderbookClient {
    chain_specific_config: BTreeMap<u64, ChainSpecificConfig>,
}

impl OrderbookClient {
    // /// Create a new OrderbookClient from a YAML configuration string
    // pub fn new(yaml_config: &str) -> Result<Self, OrderbookClientError>;

    /// Create a new OrderbookClient from a pre-parsed OrderbookYaml
    pub fn from_yaml(yaml: OrderbookYaml) -> Self {
        let chain_specific_config = BTreeMap::new();
        OrderbookClient {
            chain_specific_config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rain_orderbook_app_settings::yaml::YamlParsable;

    #[test]
    fn test_from_yaml() {
        let orderbook_yaml =
            OrderbookYaml::new(vec![YAML_WITHOUT_OPTIONAL_FIELDS.to_string()], false).unwrap();
    }

    const YAML_WITHOUT_OPTIONAL_FIELDS: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
        arbitrum:
            rpc: https://arbitrum.infura.io
            chain-id: 42161
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
        arbitrum: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        orderbook1:
            address: 0x1234567890abcdef
    tokens:
        token1:
            network: mainnet
            address: 0x2345678901abcdef
        token2:
            network: arbitrum
            address: 0x2345678901abcdef
    deployers:
        deployer1:
            address: 0x3456789012abcdef
    "#;
}
