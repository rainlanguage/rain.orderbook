// pub mod accounts;
// pub mod deployer;
// pub mod metaboards;
// pub mod orderbook_entry;
// pub mod sentry;
// pub mod subgraphs;
// pub mod token;

use super::*;
use crate::Network;
// use accounts::AccountsYaml;
// use deployer::DeployerYaml;
// use metaboards::MetaboardsYaml;
// use orderbook_entry::OrderbookEntryYaml;
// use sentry::SentryYaml;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYamlEmitter;
// use subgraphs::SubgraphsYaml;
// use token::TokenYaml;

#[derive(Debug, Clone)]
pub struct OrderbookYaml {
    pub document: Arc<RwLock<StrictYaml>>,
}

impl OrderbookYaml {
    pub fn new(source: String, validate: bool) -> Result<Self, YamlError> {
        let docs = StrictYamlLoader::load_from_str(&source)?;
        if docs.is_empty() {
            return Err(YamlError::EmptyFile);
        }
        let doc = docs[0].clone();
        let document = Arc::new(RwLock::new(doc));

        if validate {
            Network::parse_networks_from_yaml(document.clone())?;
            //     SubgraphsYaml::try_from_string(&source)?;
            //     MetaboardsYaml::try_from_string(&source)?;
            //     OrderbookEntryYaml::try_from_string(&source)?;
            //     TokenYaml::try_from_string(&source)?;
            //     DeployerYaml::try_from_string(&source)?;
            //     AccountsYaml::try_from_string(&source)?;
            //     SentryYaml::try_from_string(&source)?;
        }
        Ok(OrderbookYaml { document })
    }

    pub fn get_yaml_string(&self) -> Result<String, YamlError> {
        let document = self.document.read().unwrap();
        let mut out_str = String::new();
        let mut emitter = StrictYamlEmitter::new(&mut out_str);
        emitter.dump(&document)?;
        Ok(out_str)
    }

    pub fn get_network_keys(&self) -> Result<Vec<String>, YamlError> {
        let networks = Network::parse_networks_from_yaml(self.document.clone())?;
        Ok(networks.keys().cloned().collect())
    }
    pub fn get_network<'a>(&'a self, key: &str) -> Result<Network, YamlError> {
        let networks = Network::parse_networks_from_yaml(self.document.clone())?;
        let network = networks
            .get(key)
            .ok_or(YamlError::KeyNotFound(key.to_string()))?;
        Ok(network.clone())
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    const FULL_YAML: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
        secondary: https://api.thegraph.com/subgraphs/name/abc
    metaboards:
        board1: https://meta.example.com/board1
        board2: https://meta.example.com/board2
    orderbooks:
        orderbook1:
            address: 0x1234567890abcdef
            network: mainnet
            subgraph: main
            label: Primary Orderbook
    tokens:
        token1:
            network: mainnet
            address: 0x2345678901abcdef
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
    deployers:
        deployer1:
            address: 0x3456789012abcdef
            network: mainnet
            label: Main Deployer
    accounts:
        admin: 0x4567890123abcdef
        user: 0x5678901234abcdef
    sentry: true
    "#;

    const YAML_WITHOUT_OPTIONAL_FIELDS: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        orderbook1:
            address: 0x1234567890abcdef
    tokens:
        token1:
            network: mainnet
            address: 0x2345678901abcdef
    deployers:
        deployer1:
            address: 0x3456789012abcdef
    "#;

    #[test]
    fn test_full_yaml() {
        let ob_yaml = OrderbookYaml::new(FULL_YAML.to_string(), false).unwrap();

        assert_eq!(ob_yaml.get_network_keys().unwrap().len(), 1);
        let network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.label, Some("Ethereum Mainnet".to_string()));
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.currency, Some("ETH".to_string()));

        assert!(OrderbookYaml::new(YAML_WITHOUT_OPTIONAL_FIELDS.to_string(), true).is_ok());
    }

    #[test]
    fn test_update_network_rpc() {
        let ob_yaml = OrderbookYaml::new(FULL_YAML.to_string(), false).unwrap();

        let mut network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );

        network
            .update_rpc("https://some-random-rpc-address.com")
            .unwrap();

        let network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpc,
            Url::parse("https://some-random-rpc-address.com").unwrap()
        );
    }
}
