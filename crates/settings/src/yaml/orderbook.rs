use super::*;
use crate::{
    metaboard::MetaboardCfg, raindex_version::RaindexVersion, sentry::Sentry,
    subgraph::SubgraphCfg, DeployerCfg, NetworkCfg, OrderbookCfg, TokenCfg,
};
use alloy::primitives::Address;
use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
    Deserialize,
};
use std::{
    fmt,
    sync::{Arc, RwLock},
};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct OrderbookYaml {
    #[cfg_attr(target_family = "wasm", tsify(type = "string[]"))]
    pub documents: Vec<Arc<RwLock<StrictYaml>>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderbookYaml);

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
            NetworkCfg::parse_all_from_yaml(documents.clone(), None)?;
            TokenCfg::parse_all_from_yaml(documents.clone(), None)?;
            SubgraphCfg::parse_all_from_yaml(documents.clone(), None)?;
            OrderbookCfg::parse_all_from_yaml(documents.clone(), None)?;
            DeployerCfg::parse_all_from_yaml(documents.clone(), None)?;
            MetaboardCfg::parse_all_from_yaml(documents.clone(), None)?;
        }

        Ok(OrderbookYaml { documents })
    }

    fn from_documents(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Self {
        OrderbookYaml { documents }
    }
}

impl OrderbookYaml {
    pub fn get_network_keys(&self) -> Result<Vec<String>, YamlError> {
        let networks = NetworkCfg::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(networks.keys().cloned().collect())
    }
    pub fn get_network(&self, key: &str) -> Result<NetworkCfg, YamlError> {
        NetworkCfg::parse_from_yaml(self.documents.clone(), key, None)
    }

    pub fn get_token_keys(&self) -> Result<Vec<String>, YamlError> {
        let tokens = TokenCfg::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(tokens.keys().cloned().collect())
    }
    pub fn get_token(&self, key: &str) -> Result<TokenCfg, YamlError> {
        TokenCfg::parse_from_yaml(self.documents.clone(), key, None)
    }

    pub fn get_subgraph_keys(&self) -> Result<Vec<String>, YamlError> {
        let subgraphs = SubgraphCfg::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(subgraphs.keys().cloned().collect())
    }
    pub fn get_subgraph(&self, key: &str) -> Result<SubgraphCfg, YamlError> {
        SubgraphCfg::parse_from_yaml(self.documents.clone(), key, None)
    }

    pub fn get_orderbook_keys(&self) -> Result<Vec<String>, YamlError> {
        let orderbooks = OrderbookCfg::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(orderbooks.keys().cloned().collect())
    }
    pub fn get_orderbook(&self, key: &str) -> Result<OrderbookCfg, YamlError> {
        OrderbookCfg::parse_from_yaml(self.documents.clone(), key, None)
    }
    pub fn get_orderbook_by_address(&self, address: Address) -> Result<OrderbookCfg, YamlError> {
        let orderbooks = OrderbookCfg::parse_all_from_yaml(self.documents.clone(), None)?;
        for (_, orderbook) in orderbooks {
            if orderbook.address == address {
                return Ok(orderbook);
            }
        }
        Err(YamlError::KeyNotFound(address.to_string()))
    }

    pub fn get_metaboard_keys(&self) -> Result<Vec<String>, YamlError> {
        let metaboards = MetaboardCfg::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(metaboards.keys().cloned().collect())
    }
    pub fn get_metaboard(&self, key: &str) -> Result<MetaboardCfg, YamlError> {
        MetaboardCfg::parse_from_yaml(self.documents.clone(), key, None)
    }
    pub fn add_metaboard(&self, key: &str, value: &str) -> Result<(), YamlError> {
        MetaboardCfg::add_record_to_yaml(self.documents[0].clone(), key, value)
    }

    pub fn get_deployer_keys(&self) -> Result<Vec<String>, YamlError> {
        let deployers = DeployerCfg::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(deployers.keys().cloned().collect())
    }
    pub fn get_deployer(&self, key: &str) -> Result<DeployerCfg, YamlError> {
        DeployerCfg::parse_from_yaml(self.documents.clone(), key, None)
    }

    pub fn get_sentry(&self) -> Result<bool, YamlError> {
        let value = Sentry::parse_from_yaml_optional(self.documents[0].clone())?;
        Ok(value.map_or(false, |v| v == "true"))
    }

    pub fn get_raindex_version(&self) -> Result<Option<String>, YamlError> {
        let value = RaindexVersion::parse_from_yaml_optional(self.documents[0].clone())?;
        Ok(value)
    }
}

impl Serialize for OrderbookYaml {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.documents.len()))?;
        for doc in &self.documents {
            let yaml_str = Self::get_yaml_string(doc.clone()).map_err(serde::ser::Error::custom)?;
            seq.serialize_element(&yaml_str)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for OrderbookYaml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderbookYamlVisitor;

        impl<'de> Visitor<'de> for OrderbookYamlVisitor {
            type Value = OrderbookYaml;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of YAML documents as strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut documents = Vec::new();

                while let Some(doc_str) = seq.next_element::<String>()? {
                    let docs =
                        StrictYamlLoader::load_from_str(&doc_str).map_err(de::Error::custom)?;
                    if docs.is_empty() {
                        return Err(de::Error::custom("Empty YAML document"));
                    }
                    let doc = docs[0].clone();
                    documents.push(Arc::new(RwLock::new(doc)));
                }

                Ok(OrderbookYaml { documents })
            }
        }

        deserializer.deserialize_seq(OrderbookYamlVisitor)
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
    raindex-version: 1.0.0
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
        assert_eq!(
            NetworkCfg::parse_rpc(ob_yaml.documents.clone(), "mainnet").unwrap(),
            Url::parse("https://mainnet.infura.io").unwrap()
        );

        assert_eq!(ob_yaml.get_token_keys().unwrap().len(), 1);
        let token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(token.decimals, Some(18));
        assert_eq!(token.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token.symbol, Some("WETH".to_string()));
        assert_eq!(
            TokenCfg::parse_network_key(ob_yaml.documents.clone(), "token1").unwrap(),
            "mainnet"
        );

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
        assert_eq!(
            OrderbookCfg::parse_network_key(ob_yaml.documents.clone(), "orderbook1").unwrap(),
            "mainnet"
        );
        let orderbook_by_address = ob_yaml
            .get_orderbook_by_address(
                Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
            )
            .unwrap();
        assert_eq!(orderbook_by_address, orderbook);

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
        assert_eq!(
            DeployerCfg::parse_network_key(ob_yaml.documents.clone(), "deployer1").unwrap(),
            "mainnet"
        );

        assert!(ob_yaml.get_sentry().unwrap());

        assert_eq!(
            ob_yaml.get_raindex_version().unwrap(),
            Some("1.0.0".to_string())
        );
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
    fn test_add_token_to_yaml() {
        let yaml = r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
"#;
        let ob_yaml = OrderbookYaml::new(vec![yaml.to_string()], false).unwrap();

        TokenCfg::add_record_to_yaml(
            ob_yaml.documents.clone(),
            "test-token",
            "mainnet",
            "0x0000000000000000000000000000000000000001",
            Some("18"),
            Some("Test Token"),
            Some("TTK"),
        )
        .unwrap();

        let token = ob_yaml.get_token("test-token").unwrap();
        assert_eq!(token.key, "test-token");
        assert_eq!(token.network.key, "mainnet");
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        assert_eq!(token.decimals, Some(18));
        assert_eq!(token.label, Some("Test Token".to_string()));
        assert_eq!(token.symbol, Some("TTK".to_string()));
    }

    #[test]
    fn test_remove_token_from_yaml() {
        let ob_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()], false).unwrap();

        assert!(ob_yaml.get_token("token1").is_ok());
        TokenCfg::remove_record_from_yaml(ob_yaml.documents.clone(), "token1").unwrap();
        assert!(ob_yaml.get_token("token1").is_err());
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
