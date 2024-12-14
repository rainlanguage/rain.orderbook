use super::*;
use crate::{
    metaboard::YamlMetaboard, subgraph::YamlSubgraph, Metaboard, Network, Orderbook, Subgraph,
    Token,
};
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
        Network::parse_from_yaml(self.document.clone(), key)
    }

    pub fn get_token_keys(&self) -> Result<Vec<String>, YamlError> {
        let tokens = Token::parse_all_from_yaml(self.document.clone())?;
        Ok(tokens.keys().cloned().collect())
    }
    pub fn get_token(&self, key: &str) -> Result<Token, YamlError> {
        Token::parse_from_yaml(self.document.clone(), key)
    }

    pub fn get_subgraph_keys(&self) -> Result<Vec<String>, YamlError> {
        let subgraphs = YamlSubgraph::parse_all_from_yaml(self.document.clone())?;
        Ok(subgraphs.keys().cloned().collect())
    }
    pub fn get_subgraph(&self, key: &str) -> Result<Subgraph, YamlError> {
        let yaml_subgraph = YamlSubgraph::parse_from_yaml(self.document.clone(), key)?;
        Ok(yaml_subgraph.into())
    }

    pub fn get_orderbook_keys(&self) -> Result<Vec<String>, YamlError> {
        let orderbooks = Orderbook::parse_all_from_yaml(self.document.clone())?;
        Ok(orderbooks.keys().cloned().collect())
    }
    pub fn get_orderbook(&self, key: &str) -> Result<Orderbook, YamlError> {
        Orderbook::parse_from_yaml(self.document.clone(), key)
    }

    pub fn get_metaboard_keys(&self) -> Result<Vec<String>, YamlError> {
        let metaboards = YamlMetaboard::parse_all_from_yaml(self.document.clone())?;
        Ok(metaboards.keys().cloned().collect())
    }
    pub fn get_metaboard(&self, key: &str) -> Result<Metaboard, YamlError> {
        let yaml_metaboard = YamlMetaboard::parse_from_yaml(self.document.clone(), key)?;
        Ok(yaml_metaboard.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;
    use url::Url;

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
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
            subgraph: mainnet
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

    const _YAML_WITHOUT_OPTIONAL_FIELDS: &str = r#"
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

        assert_eq!(ob_yaml.get_token_keys().unwrap().len(), 1);
        let token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(token.decimals, Some(18));
        assert_eq!(token.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token.symbol, Some("WETH".to_string()));

        assert_eq!(ob_yaml.get_subgraph_keys().unwrap().len(), 2);
        let subgraph = ob_yaml.get_subgraph("mainnet").unwrap();
        assert_eq!(
            subgraph,
            Url::parse("https://api.thegraph.com/subgraphs/name/xyz").unwrap()
        );

        assert_eq!(ob_yaml.get_orderbook_keys().unwrap().len(), 1);
        let orderbook = ob_yaml.get_orderbook("orderbook1").unwrap();
        assert_eq!(
            orderbook.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(orderbook.network, network.into());
        assert_eq!(orderbook.subgraph, subgraph.into());
        assert_eq!(orderbook.label, Some("Primary Orderbook".to_string()));

        assert_eq!(ob_yaml.get_metaboard_keys().unwrap().len(), 2);
        let metaboard = ob_yaml.get_metaboard("board1").unwrap();
        assert_eq!(
            metaboard,
            Url::parse("https://meta.example.com/board1").unwrap()
        );
        let metaboard = ob_yaml.get_metaboard("board2").unwrap();
        assert_eq!(
            metaboard,
            Url::parse("https://meta.example.com/board2").unwrap()
        );
    }

    #[test]
    fn test_update_network_rpc() {
        let ob_yaml = OrderbookYaml::new(FULL_YAML.to_string(), false).unwrap();

        let mut network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );

        let network = network
            .update_rpc("https://some-random-rpc-address.com")
            .unwrap();
        assert_eq!(
            network.rpc,
            Url::parse("https://some-random-rpc-address.com").unwrap()
        );

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

        let token = token
            .update_address("0x0000000000000000000000000000000000000001")
            .unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );

        let token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
    }
}
