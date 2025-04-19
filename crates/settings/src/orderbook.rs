use crate::yaml::FieldErrorKind;
use crate::*;
use alloy::primitives::hex::FromHexError;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use subgraph::SubgraphCfg;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::context::Context;
use yaml::{
    default_document, optional_string, require_hash, require_string, YamlError, YamlParsableHash,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct OrderbookCfgSource {
    pub key: String,
    pub address: String,
    pub network: String,
    pub subgraph: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderbookCfgSource);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct OrderbookCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub network: Arc<NetworkCfg>,
    pub subgraph: Arc<SubgraphCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderbookCfg);

impl OrderbookCfg {
    pub fn validate_address(address: &str) -> Result<Address, ParseOrderbookCfgError> {
        Address::from_str(address).map_err(ParseOrderbookCfgError::AddressParseError)
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

impl YamlParsableHash for OrderbookCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut orderbooks = HashMap::new();

        let networks = NetworkCfg::parse_all_from_yaml(documents.clone(), context)?;
        let subgraphs = SubgraphCfg::parse_all_from_yaml(documents.clone(), context)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orderbooks_hash) = require_hash(&document_read, Some("orderbooks"), None) {
                for (key_yaml, orderbook_yaml) in orderbooks_hash {
                    let orderbook_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("orderbook '{}'", orderbook_key);

                    let address_str =
                        require_string(orderbook_yaml, Some("address"), Some(location.clone()))?;
                    let address = OrderbookCfg::validate_address(&address_str).map_err(|e| {
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

                    let orderbook = OrderbookCfg {
                        document: document.clone(),
                        key: orderbook_key.clone(),
                        address,
                        network: Arc::new(network.clone()),
                        subgraph: Arc::new(subgraph.clone()),
                        label,
                    };

                    if orderbooks.contains_key(&orderbook_key) {
                        return Err(YamlError::KeyShadowing(
                            orderbook_key,
                            "orderbooks".to_string(),
                        ));
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

impl Default for OrderbookCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::ZERO,
            network: Arc::new(NetworkCfg::default()),
            subgraph: Arc::new(SubgraphCfg::default()),
            label: None,
        }
    }
}
impl PartialEq for OrderbookCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.address == other.address
            && self.network == other.network
            && self.subgraph == other.subgraph
            && self.label == other.label
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseOrderbookCfgError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Network not found for Orderbook: {0}")]
    NetworkNotFoundError(String),
    #[error("Subgraph not found: {0}")]
    SubgraphNotFoundError(String),
}

impl ParseOrderbookCfgError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseOrderbookCfgError::AddressParseError(err) =>
                format!("The orderbook address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseOrderbookCfgError::NetworkNotFoundError(network) =>
                format!("The network '{}' specified for this orderbook was not found in your YAML configuration. Please define this network or use an existing one.", network),
            ParseOrderbookCfgError::SubgraphNotFoundError(subgraph) =>
                format!("The subgraph '{}' specified for this orderbook was not found in your YAML configuration. Please define this subgraph or use an existing one.", subgraph),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strict_yaml_rust::StrictYamlLoader;

    fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
    }

    #[test]
    fn test_parse_orderbooks_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'networks' in root"
        );

        let yaml = r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
test: test
"#;
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("subgraphs".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'subgraphs' in root"
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
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("orderbooks".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'orderbooks' in root"
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
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("address".to_string()),
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'address' in orderbook 'TestOrderbook'"
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
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
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
        assert_eq!(error.to_readable_msg(), "Invalid value for field 'network' in orderbook 'TestOrderbook': Network 'TestNetwork' not found");

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
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
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
        assert_eq!(error.to_readable_msg(), "Invalid value for field 'subgraph' in orderbook 'TestOrderbook': Subgraph 'TestSubgraph' not found");
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
        let orderbooks = OrderbookCfg::parse_all_from_yaml(documents, None).unwrap();

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
        let error = OrderbookCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateOrderbook".to_string(), "orderbooks".to_string())
        );
        assert_eq!(error.to_readable_msg(), "The key 'DuplicateOrderbook' is defined multiple times in your YAML configuration at orderbooks");
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
        let network_key = OrderbookCfg::parse_network_key(documents, "mainnet").unwrap();
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
        let network_key = OrderbookCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
orderbooks: test
"#;
        let error =
            OrderbookCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
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
        assert_eq!(
            error.to_readable_msg(),
            "Field 'orderbooks' in root must be a map"
        );

        let yaml = r#"
orderbooks:
  - test
"#;
        let error =
            OrderbookCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
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
        assert_eq!(
            error.to_readable_msg(),
            "Field 'orderbooks' in root must be a map"
        );

        let yaml = r#"
orderbooks:
  - test: test
"#;
        let error =
            OrderbookCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
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
        assert_eq!(
            error.to_readable_msg(),
            "Field 'orderbooks' in root must be a map"
        );
    }
}
