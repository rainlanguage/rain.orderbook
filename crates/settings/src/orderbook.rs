use crate::yaml::FieldErrorKind;
use crate::*;
use alloy::primitives::hex::FromHexError;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use subgraph::Subgraph;
use thiserror::Error;
use typeshare::typeshare;
use yaml::context::Context;
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

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        orderbook_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orderbooks_hash) = require_hash(&document_read, Some("orderbooks"), None) {
                if let Some(orderbook_yaml) =
                    orderbooks_hash.get(&StrictYaml::String(orderbook_key.to_string()))
                {
                    return require_string(orderbook_yaml, Some("network"), None)
                        .or_else(|_| Ok(orderbook_key.to_string()));
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "orderbooks".to_string(),
                        expected: "a map".to_string(),
                    },
                    location: "root".to_string(),
                });
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("network for orderbook '{}'", orderbook_key)),
            location: "root".to_string(),
        })
    }
}

impl YamlParsableHash for Orderbook {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut orderbooks = HashMap::new();

        let networks = Network::parse_all_from_yaml(documents.clone(), None)?;
        let subgraphs = Subgraph::parse_all_from_yaml(documents.clone(), None)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orderbooks_hash) = require_hash(&document_read, Some("orderbooks"), None) {
                for (key_yaml, orderbook_yaml) in orderbooks_hash {
                    let orderbook_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("orderbook '{}'", orderbook_key);

                    let address_str =
                        require_string(orderbook_yaml, Some("address"), Some(location.clone()))?;
                    let address = Orderbook::validate_address(&address_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let network_name = match optional_string(orderbook_yaml, "network") {
                        Some(network_name) => network_name,
                        None => orderbook_key.clone(),
                    };
                    let network = networks
                        .get(&network_name)
                        .ok_or_else(|| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "network".to_string(),
                                reason: format!("Network '{}' not found", network_name),
                            },
                            location: location.clone(),
                        })?;

                    let subgraph_name = match optional_string(orderbook_yaml, "subgraph") {
                        Some(subgraph_name) => subgraph_name,
                        None => orderbook_key.clone(),
                    };
                    let subgraph =
                        subgraphs
                            .get(&subgraph_name)
                            .ok_or_else(|| YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "subgraph".to_string(),
                                    reason: format!("Subgraph '{}' not found", subgraph_name),
                                },
                                location: location.clone(),
                            })?;

                    let label = optional_string(orderbook_yaml, "label");

                    let orderbook = Orderbook {
                        document: document.clone(),
                        key: orderbook_key.clone(),
                        address,
                        network: Arc::new(network.clone()),
                        subgraph: Arc::new(subgraph.clone()),
                        label,
                    };

                    if orderbooks.contains_key(&orderbook_key) {
                        return Err(YamlError::KeyShadowing(orderbook_key));
                    }
                    orderbooks.insert(orderbook_key, orderbook);
                }
            }
        }

        if orderbooks.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("orderbooks".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(orderbooks)
    }
}

impl Default for Orderbook {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::ZERO,
            network: Arc::new(Network::default()),
            subgraph: Arc::new(Subgraph::default()),
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
        let error = Orderbook::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            }
        );

        let yaml = r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
test: test
"#;
        let error = Orderbook::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("subgraphs".to_string()),
                location: "root".to_string(),
            }
        );

        let yaml = r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
test: test
"#;
        let error = Orderbook::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("orderbooks".to_string()),
                location: "root".to_string(),
            }
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
"#;
        let error = Orderbook::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("address".to_string()),
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );

        let yaml = r#"
networks:
    SomeNetwork:
        rpc: https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let error = Orderbook::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "network".to_string(),
                    reason: "Network 'TestNetwork' not found".to_string(),
                },
                location: "orderbook 'TestOrderbook'".to_string(),
            }
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
        let error = Orderbook::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "subgraph".to_string(),
                    reason: "Subgraph 'TestSubgraph' not found".to_string(),
                },
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_orderbooks_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    OrderbookOne:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
"#;
        let yaml_two = r#"
orderbooks:
    OrderbookTwo:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let orderbooks = Orderbook::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(orderbooks.len(), 2);
        assert!(orderbooks.contains_key("OrderbookOne"));
        assert!(orderbooks.contains_key("OrderbookTwo"));

        assert_eq!(
            orderbooks.get("OrderbookOne").unwrap().address.to_string(),
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(
            orderbooks.get("OrderbookTwo").unwrap().address.to_string(),
            "0x0987654321098765432109876543210987654321"
        );
    }

    #[test]
    fn test_parse_orderbooks_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    DuplicateOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
"#;
        let yaml_two = r#"
orderbooks:
    DuplicateOrderbook:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = Orderbook::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateOrderbook".to_string())
        );
    }

    #[test]
    fn test_parse_orderbook_from_yaml_network_key() {
        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
orderbooks:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
"#;

        let documents = vec![get_document(yaml)];
        let network_key = Orderbook::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
orderbooks:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
"#;
        let documents = vec![get_document(yaml)];
        let network_key = Orderbook::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
orderbooks: test
"#;
        let error = Orderbook::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "orderbooks".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );

        let yaml = r#"
orderbooks:
  - test
"#;
        let error = Orderbook::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "orderbooks".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );

        let yaml = r#"
orderbooks:
  - test: test
"#;
        let error = Orderbook::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "orderbooks".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
    }
}
