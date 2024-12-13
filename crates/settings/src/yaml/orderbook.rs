use super::*;
use crate::{Network, Token};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYamlEmitter;

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
            Network::parse_all_from_yaml(document.clone())?;
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
        let networks = Network::parse_all_from_yaml(self.document.clone())?;
        Ok(networks.keys().cloned().collect())
    }
    pub fn get_network(&self, key: &str) -> Result<Network, YamlError> {
        let networks = Network::parse_all_from_yaml(self.document.clone())?;
        let network = networks
            .get(key)
            .ok_or(YamlError::KeyNotFound(key.to_string()))?;
        Ok(network.clone())
    }

    pub fn get_token_keys(&self) -> Result<Vec<String>, YamlError> {
        let tokens = Token::parse_all_from_yaml(self.document.clone())?;
        Ok(tokens.keys().cloned().collect())
    }
    pub fn get_token(&self, key: &str) -> Result<Token, YamlError> {
        let tokens = Token::parse_all_from_yaml(self.document.clone())?;
        let token = tokens
            .get(key)
            .ok_or(YamlError::KeyNotFound(key.to_string()))?;
        Ok(token.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::primitives::Address;
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
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
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

    #[test]
    fn test_update_token_address() {
        let ob_yaml = OrderbookYaml::new(FULL_YAML.to_string(), false).unwrap();

        let mut token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );

        token
            .update_address("0x0000000000000000000000000000000000000001")
            .unwrap();

        let token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
    }
}
