use super::*;
use crate::{
    metaboard::Metaboard, sentry::Sentry, subgraph::Subgraph, Deployer, Network, Orderbook, Token,
};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct OrderbookYaml {
    pub documents: Vec<Arc<RwLock<StrictYaml>>>,
}

impl YamlParsable for OrderbookYaml {
    fn new(sources: Vec<String>, validate: bool) -> Result<Self, YamlError> {
        let mut documents = Vec::new();

        for source in sources {
            let docs = StrictYamlLoader::load_from_str(&source)?;
            if docs.is_empty() {
                return Err(YamlError::EmptyFile);
            }
            let doc = docs[0].clone();
            let document = Arc::new(RwLock::new(doc));

            documents.push(document);
        }

        if validate {
            Network::parse_all_from_yaml(documents.clone())?;
        }

        Ok(OrderbookYaml { documents })
    }
}

impl OrderbookYaml {
    pub fn get_network_keys(&self) -> Result<Vec<String>, YamlError> {
        let networks = Network::parse_all_from_yaml(self.documents.clone())?;
        Ok(networks.keys().cloned().collect())
    }
    pub fn get_network(&self, key: &str) -> Result<Network, YamlError> {
        Network::parse_from_yaml(self.documents.clone(), key)
    }

    pub fn get_token_keys(&self) -> Result<Vec<String>, YamlError> {
        let tokens = Token::parse_all_from_yaml(self.documents.clone())?;
        Ok(tokens.keys().cloned().collect())
    }
    pub fn get_token(&self, key: &str) -> Result<Token, YamlError> {
        Token::parse_from_yaml(self.documents.clone(), key)
    }

    pub fn get_subgraph_keys(&self) -> Result<Vec<String>, YamlError> {
        let subgraphs = Subgraph::parse_all_from_yaml(self.documents.clone())?;
        Ok(subgraphs.keys().cloned().collect())
    }
    pub fn get_subgraph(&self, key: &str) -> Result<Subgraph, YamlError> {
        Subgraph::parse_from_yaml(self.documents.clone(), key)
    }

    pub fn get_orderbook_keys(&self) -> Result<Vec<String>, YamlError> {
        let orderbooks = Orderbook::parse_all_from_yaml(self.documents.clone())?;
        Ok(orderbooks.keys().cloned().collect())
    }
    pub fn get_orderbook(&self, key: &str) -> Result<Orderbook, YamlError> {
        Orderbook::parse_from_yaml(self.documents.clone(), key)
    }

    pub fn get_metaboard_keys(&self) -> Result<Vec<String>, YamlError> {
        let metaboards = Metaboard::parse_all_from_yaml(self.documents.clone())?;
        Ok(metaboards.keys().cloned().collect())
    }
    pub fn get_metaboard(&self, key: &str) -> Result<Metaboard, YamlError> {
        Metaboard::parse_from_yaml(self.documents.clone(), key)
    }
    pub fn add_metaboard(&self, key: &str, value: &str) -> Result<(), YamlError> {
        Metaboard::add_record_to_yaml(self.documents[0].clone(), key, value)
    }

    pub fn get_deployer_keys(&self) -> Result<Vec<String>, YamlError> {
        let deployers = Deployer::parse_all_from_yaml(self.documents.clone())?;
        Ok(deployers.keys().cloned().collect())
    }
    pub fn get_deployer(&self, key: &str) -> Result<Deployer, YamlError> {
        Deployer::parse_from_yaml(self.documents.clone(), key)
    }

    pub fn get_sentry(&self) -> Result<bool, YamlError> {
        let value = Sentry::parse_from_yaml_optional(self.documents[0].clone())?;
        Ok(value.map_or(false, |v| v == "true"))
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
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
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
        let ob_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()], false).unwrap();

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
            subgraph.url,
            Url::parse("https://api.thegraph.com/subgraphs/name/xyz").unwrap()
        );

        assert_eq!(ob_yaml.get_orderbook_keys().unwrap().len(), 1);
        let orderbook = ob_yaml.get_orderbook("orderbook1").unwrap();
        assert_eq!(
            orderbook.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(orderbook.network, network.clone().into());
        assert_eq!(orderbook.subgraph, subgraph.into());
        assert_eq!(orderbook.label, Some("Primary Orderbook".to_string()));

        assert_eq!(ob_yaml.get_metaboard_keys().unwrap().len(), 2);
        assert_eq!(
            ob_yaml.get_metaboard("board1").unwrap().url,
            Url::parse("https://meta.example.com/board1").unwrap()
        );
        assert_eq!(
            ob_yaml.get_metaboard("board2").unwrap().url,
            Url::parse("https://meta.example.com/board2").unwrap()
        );

        assert_eq!(ob_yaml.get_deployer_keys().unwrap().len(), 1);
        let deployer = ob_yaml.get_deployer("deployer1").unwrap();
        assert_eq!(
            deployer.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(deployer.network, network.into());

        assert!(ob_yaml.get_sentry().unwrap());
    }

    #[test]
    fn test_update_network_rpc() {
        let ob_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()], false).unwrap();

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
        let ob_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()], false).unwrap();

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

    #[test]
    fn test_add_metaboard_to_yaml() {
        let yaml = r#"
test: test
"#;
        let ob_yaml = OrderbookYaml::new(vec![yaml.to_string()], false).unwrap();

        ob_yaml
            .add_metaboard("test-metaboard", "https://test-metaboard.com")
            .unwrap();

        assert_eq!(
            ob_yaml.get_metaboard_keys().unwrap(),
            vec!["test-metaboard".to_string()]
        );
        assert_eq!(
            ob_yaml.get_metaboard("test-metaboard").unwrap().url,
            Url::parse("https://test-metaboard.com").unwrap()
        );
    }
}
