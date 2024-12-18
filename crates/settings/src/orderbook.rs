use crate::*;
use alloy::primitives::hex::FromHexError;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use subgraph::YamlSubgraph;
use thiserror::Error;
use typeshare::typeshare;
use yaml::{
    default_document, optional_string, require_hash, require_string, YamlError, YamlParsableHash,
};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Orderbook {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "string"))]
    pub address: Address,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    #[typeshare(typescript(type = "string"))]
    pub subgraph: Arc<Subgraph>,
    pub label: Option<String>,
}

impl Orderbook {
    pub fn validate_address(address: &str) -> Result<Address, ParseOrderbookConfigSourceError> {
        Address::from_str(address).map_err(ParseOrderbookConfigSourceError::AddressParseError)
    }
}

impl YamlParsableHash for Orderbook {
    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let orderbooks_hash = require_hash(
            &document_read,
            Some("orderbooks"),
            Some("missing field: orderbooks".to_string()),
        )?;

        orderbooks_hash
            .into_iter()
            .map(|(key_yaml, orderbook_yaml)| {
                let orderbook_key = key_yaml.as_str().unwrap_or_default().to_string();

                let address = Orderbook::validate_address(&require_string(
                    orderbook_yaml,
                    Some("address"),
                    Some(format!(
                        "address string missing in orderbook: {orderbook_key}"
                    )),
                )?)?;

                let network_name = match optional_string(orderbook_yaml, "network") {
                    Some(network_name) => network_name,
                    None => orderbook_key.clone(),
                };
                let network = Network::parse_from_yaml(document.clone(), &network_name)?;

                let subgraph_name = match optional_string(orderbook_yaml, "subgraph") {
                    Some(subgraph_name) => subgraph_name,
                    None => orderbook_key.clone(),
                };
                let subgraph = YamlSubgraph::parse_from_yaml(document.clone(), &subgraph_name)?;
                let subgraph = Arc::new(subgraph.into());

                let label = optional_string(orderbook_yaml, "label");

                let orderbook = Orderbook {
                    document: document.clone(),
                    key: orderbook_key.clone(),
                    address,
                    network: Arc::new(network),
                    subgraph,
                    label,
                };

                Ok((orderbook_key, orderbook))
            })
            .collect()
    }
}

impl Default for Orderbook {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::ZERO,
            network: Arc::new(Network::default()),
            subgraph: Arc::new(Subgraph::parse("https://subgraph.com").unwrap()),
            label: None,
        }
    }
}
impl PartialEq for Orderbook {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.address == other.address
            && self.network == other.network
            && self.subgraph == other.subgraph
            && self.label == other.label
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseOrderbookConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Network not found for Orderbook: {0}")]
    NetworkNotFoundError(String),
    #[error("Subgraph not found: {0}")]
    SubgraphNotFoundError(String),
}

impl OrderbookConfigSource {
    pub fn try_into_orderbook(
        self,
        name: String,
        networks: &HashMap<String, Arc<Network>>,
        subgraphs: &HashMap<String, Arc<Subgraph>>,
    ) -> Result<Orderbook, ParseOrderbookConfigSourceError> {
        let network_ref = match self.network {
            Some(network_name) => networks
                .get(&network_name)
                .ok_or(ParseOrderbookConfigSourceError::NetworkNotFoundError(
                    network_name.clone(),
                ))
                .map(Arc::clone)?,
            None => networks
                .get(&name)
                .ok_or(ParseOrderbookConfigSourceError::NetworkNotFoundError(
                    name.clone(),
                ))
                .map(Arc::clone)?,
        };

        let subgraph_ref = match self.subgraph {
            Some(subgraph_name) => subgraphs
                .get(&subgraph_name)
                .ok_or(ParseOrderbookConfigSourceError::SubgraphNotFoundError(
                    subgraph_name.clone(),
                ))
                .map(Arc::clone)?,
            None => subgraphs
                .get(&name)
                .ok_or(ParseOrderbookConfigSourceError::SubgraphNotFoundError(
                    name.clone(),
                ))
                .map(Arc::clone)?,
        };

        Ok(Orderbook {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: name,
            address: self.address,
            network: network_ref,
            subgraph: subgraph_ref,
            label: self.label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;
    use alloy::primitives::Address;
    use strict_yaml_rust::StrictYamlLoader;

    fn setup() -> (
        HashMap<String, Arc<Network>>,
        HashMap<String, Arc<Subgraph>>,
    ) {
        let network = mock_network();
        let subgraph = mock_subgraph();

        let mut networks = HashMap::new();
        networks.insert("TestNetwork".to_string(), network);

        let mut subgraphs = HashMap::new();
        subgraphs.insert("TestSubgraph".to_string(), subgraph);

        (networks, subgraphs)
    }

    #[test]
    fn test_orderbook_creation_success() {
        let (networks, subgraphs) = setup();
        let address = "0x1234567890123456789012345678901234567890"
            .parse::<Address>()
            .unwrap();
        let orderbook_string = OrderbookConfigSource {
            address,
            network: Some("TestNetwork".to_string()),
            subgraph: Some("TestSubgraph".to_string()),
            label: Some("TestLabel".to_string()),
        };

        let orderbook =
            orderbook_string.try_into_orderbook("TestName".to_string(), &networks, &subgraphs);

        assert!(orderbook.is_ok());
        let orderbook = orderbook.unwrap();

        assert_eq!(orderbook.address, address);
        assert_eq!(
            Arc::as_ptr(&orderbook.network),
            Arc::as_ptr(networks.get("TestNetwork").unwrap())
        );
        assert_eq!(
            Arc::as_ptr(&orderbook.subgraph),
            Arc::as_ptr(subgraphs.get("TestSubgraph").unwrap())
        );
        assert_eq!(orderbook.label, Some("TestLabel".to_string()));
    }

    #[test]
    fn test_orderbook_creation_with_missing_network() {
        let (networks, subgraphs) = setup();
        let orderbook_string = OrderbookConfigSource {
            address: Address::random(),
            network: Some("NonExistingNetwork".to_string()),
            subgraph: Some("TestSubgraph".to_string()),
            label: None,
        };

        let result =
            orderbook_string.try_into_orderbook("TestName".to_string(), &networks, &subgraphs);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseOrderbookConfigSourceError::NetworkNotFoundError("NonExistingNetwork".to_string())
        );
    }

    #[test]
    fn test_orderbook_creation_with_missing_subgraph() {
        let (networks, subgraphs) = setup();
        let orderbook_string = OrderbookConfigSource {
            address: Address::random(),
            network: Some("TestNetwork".to_string()),
            subgraph: Some("NonExistingSubgraph".to_string()),
            label: None,
        };

        let result =
            orderbook_string.try_into_orderbook("TestName".to_string(), &networks, &subgraphs);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseOrderbookConfigSourceError::SubgraphNotFoundError(
                "NonExistingSubgraph".to_string()
            )
        );
    }

    fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
    }

    #[test]
    fn test_parse_orderbooks_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Orderbook::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: orderbooks".to_string())
        );

        let yaml = r#"
orderbooks:
    TestOrderbook:
"#;
        let error = Orderbook::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address string missing in orderbook: TestOrderbook".to_string())
        );

        let yaml = r#"
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let error = Orderbook::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: networks".to_string())
        );

        let yaml = r#"
networks:
    SomeNetwork:
        rpc: https://rpc.com
        chain-id: 1
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let error = Orderbook::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(error, YamlError::KeyNotFound("TestNetwork".to_string()));

        let yaml = r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let error = Orderbook::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: subgraphs".to_string())
        );

        let yaml = r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
"#;
        let error = Orderbook::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(error, YamlError::KeyNotFound("TestSubgraph".to_string()));
    }
}
