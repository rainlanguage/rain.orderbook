use crate::yaml::FieldErrorKind;
use crate::*;
use alloy::primitives::hex::FromHexError;
use alloy::primitives::Address;
use local_db_remotes::LocalDbRemoteCfg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::strict_yaml::Hash;
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
pub struct OrderbookCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub network: Arc<NetworkCfg>,
    pub subgraph: Arc<SubgraphCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub local_db_remote: Option<Arc<LocalDbRemoteCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
    pub deployment_block: u64,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderbookCfg);

impl OrderbookCfg {
    pub fn validate_address(address: &str) -> Result<Address, ParseOrderbookConfigSourceError> {
        Address::from_str(address).map_err(ParseOrderbookConfigSourceError::AddressParseError)
    }

    pub fn validate_deployment_block(value: &str) -> Result<u64, ParseOrderbookConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseOrderbookConfigSourceError::DeploymentBlockParseError)
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
        let local_db_remotes = LocalDbRemoteCfg::parse_all_from_yaml(documents.clone(), context)?;

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

                    let local_db_remote_name = optional_string(orderbook_yaml, "local-db-remote");
                    let local_db_remote = if let Some(name) = local_db_remote_name {
                        local_db_remotes.get(&name).cloned()
                    } else {
                        local_db_remotes.get(&orderbook_key).cloned()
                    }
                    .map(Arc::new);

                    let deployment_block_str = require_string(
                        orderbook_yaml,
                        Some("deployment-block"),
                        Some(location.clone()),
                    )?;
                    let deployment_block = OrderbookCfg::validate_deployment_block(
                        &deployment_block_str,
                    )
                    .map_err(|e| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "deployment-block".to_string(),
                            reason: e.to_string(),
                        },
                        location: location.clone(),
                    })?;

                    let orderbook = OrderbookCfg {
                        document: document.clone(),
                        key: orderbook_key.clone(),
                        address,
                        network: Arc::new(network.clone()),
                        subgraph: Arc::new(subgraph.clone()),
                        local_db_remote,
                        label,
                        deployment_block,
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

    fn to_yaml_value(&self) -> Result<StrictYaml, YamlError> {
        let mut orderbook_yaml = Hash::new();

        orderbook_yaml.insert(
            StrictYaml::String("address".to_string()),
            StrictYaml::String(self.address.to_string()),
        );
        orderbook_yaml.insert(
            StrictYaml::String("network".to_string()),
            StrictYaml::String(self.network.key.clone()),
        );
        orderbook_yaml.insert(
            StrictYaml::String("subgraph".to_string()),
            StrictYaml::String(self.subgraph.key.clone()),
        );

        if let Some(local_db_remote) = &self.local_db_remote {
            orderbook_yaml.insert(
                StrictYaml::String("local-db-remote".to_string()),
                StrictYaml::String(local_db_remote.key.clone()),
            );
        }

        if let Some(label) = &self.label {
            orderbook_yaml.insert(
                StrictYaml::String("label".to_string()),
                StrictYaml::String(label.clone()),
            );
        }

        orderbook_yaml.insert(
            StrictYaml::String("deployment-block".to_string()),
            StrictYaml::String(self.deployment_block.to_string()),
        );

        Ok(StrictYaml::Hash(orderbook_yaml))
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
            local_db_remote: None,
            label: None,
            deployment_block: 0,
        }
    }
}
impl PartialEq for OrderbookCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.address == other.address
            && self.network == other.network
            && self.subgraph == other.subgraph
            && self.local_db_remote == other.local_db_remote
            && self.label == other.label
            && self.deployment_block == other.deployment_block
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
    #[error("Failed to parse deployment block: {0}")]
    DeploymentBlockParseError(ParseIntError),
}

impl ParseOrderbookConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseOrderbookConfigSourceError::AddressParseError(err) =>
                format!("The orderbook address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseOrderbookConfigSourceError::NetworkNotFoundError(network) =>
                format!("The network '{}' specified for this orderbook was not found in your YAML configuration. Please define this network or use an existing one.", network),
            ParseOrderbookConfigSourceError::SubgraphNotFoundError(subgraph) =>
                format!("The subgraph '{}' specified for this orderbook was not found in your YAML configuration. Please define this subgraph or use an existing one.", subgraph),
            ParseOrderbookConfigSourceError::DeploymentBlockParseError(err) =>
                format!("The deployment block in your orderbook configuration must be a valid number: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;
    use std::str::FromStr;
    use url::Url;

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
        rpcs:
            - https://rpc.com
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
        rpcs:
            - https://rpc.com
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
        rpcs:
            - https://rpc.com
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
        rpcs:
            - https://rpc.com
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
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345
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
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    OrderbookOne:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345
"#;
        let yaml_two = r#"
orderbooks:
    OrderbookTwo:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 67890
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
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    DuplicateOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345
"#;
        let yaml_two = r#"
orderbooks:
    DuplicateOrderbook:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 67890
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
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    mainnet: https://subgraph.com
orderbooks:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        local-db-remote: mainnet
        deployment-block: 12345
"#;

        let documents = vec![get_document(yaml)];
        let network_key = OrderbookCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    mainnet: https://subgraph.com
orderbooks:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        deployment-block: 12345
        local-db-remote: mainnet
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

    #[test]
    fn test_deployment_block_missing() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
"#;
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployment-block".to_string()),
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'deployment-block' in orderbook 'TestOrderbook'"
        );
    }

    #[test]
    fn test_deployment_block_valid_values() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook1:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 0
    TestOrderbook2:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 18446744073709551615
    TestOrderbook3:
        address: 0x1111111111111111111111111111111111111111
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345678
"#;
        let orderbooks = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        assert_eq!(orderbooks.len(), 3);
        assert_eq!(
            orderbooks.get("TestOrderbook1").unwrap().deployment_block,
            0
        );
        assert_eq!(
            orderbooks.get("TestOrderbook2").unwrap().deployment_block,
            18446744073709551615
        );
        assert_eq!(
            orderbooks.get("TestOrderbook3").unwrap().deployment_block,
            12345678
        );
    }

    #[test]
    fn test_deployment_block_negative_number() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: -1
"#;
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason: "Failed to parse deployment block: invalid digit found in string"
                        .to_string(),
                },
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );
    }

    #[test]
    fn test_deployment_block_too_large() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 18446744073709551616
"#;
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason:
                        "Failed to parse deployment block: number too large to fit in target type"
                            .to_string(),
                },
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );
    }

    #[test]
    fn test_deployment_block_non_numeric() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: abc123
"#;
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason: "Failed to parse deployment block: invalid digit found in string"
                        .to_string(),
                },
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );
    }

    #[test]
    fn test_deployment_block_decimal() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 123.45
"#;
        let error = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason: "Failed to parse deployment block: invalid digit found in string"
                        .to_string(),
                },
                location: "orderbook 'TestOrderbook'".to_string(),
            }
        );
    }

    #[test]
    fn test_orderbook_local_db_remote_absent_defaults_to_orderbook_key() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
local-db-remotes:
    TestOrderbook: https://example.com/localdb/TestOrderbook
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        deployment-block: 123
"#;
        let orderbooks = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let ob = orderbooks.get("TestOrderbook").unwrap();
        let remote = ob.local_db_remote.as_ref().expect("expected remote");
        assert_eq!(remote.key, "TestOrderbook");
        assert_eq!(
            remote.url.to_string(),
            "https://example.com/localdb/TestOrderbook"
        );
    }

    #[test]
    fn test_orderbook_local_db_remote_resolves() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
local-db-remotes:
    mainnet: https://example.com/localdb/mainnet
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 123
"#;
        let orderbooks = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let ob = orderbooks.get("TestOrderbook").unwrap();
        let remote = ob.local_db_remote.as_ref().expect("expected remote");
        assert_eq!(remote.key, "mainnet");
        assert_eq!(
            remote.url.to_string(),
            "https://example.com/localdb/mainnet"
        );
    }

    #[test]
    fn test_orderbook_local_db_remote_not_found() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: missing
        deployment-block: 123
"#;
        let orderbooks = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let ob = orderbooks.get("TestOrderbook").unwrap();
        assert!(ob.local_db_remote.is_none());

        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
orderbooks:
    TestOrderbook:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: missing
        deployment-block: 123
"#;
        let orderbooks = OrderbookCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let ob = orderbooks.get("TestOrderbook").unwrap();
        assert!(ob.local_db_remote.is_none());
    }

    #[test]
    fn test_to_yaml_hash_serializes_all_fields() {
        let network = Arc::new(NetworkCfg {
            document: default_document(),
            key: "mainnet".to_string(),
            rpcs: vec![Url::parse("https://rpc.mainnet.example").unwrap()],
            chain_id: 1,
            label: None,
            network_id: None,
            currency: None,
        });
        let subgraph = Arc::new(SubgraphCfg {
            document: default_document(),
            key: "mainnet".to_string(),
            url: Url::parse("https://subgraph.example/mainnet").unwrap(),
        });
        let local_db_remote = Arc::new(LocalDbRemoteCfg {
            document: default_document(),
            key: "mainnet-remote".to_string(),
            url: Url::parse("https://example.com/localdb/mainnet").unwrap(),
        });

        let mut orderbooks = HashMap::new();
        orderbooks.insert(
            "primary".to_string(),
            OrderbookCfg {
                document: default_document(),
                key: "primary".to_string(),
                address: Address::from_str("0x0000000000000000000000000000000000000001").unwrap(),
                network: Arc::clone(&network),
                subgraph: Arc::clone(&subgraph),
                local_db_remote: Some(Arc::clone(&local_db_remote)),
                label: Some("Primary Orderbook".to_string()),
                deployment_block: 12345,
            },
        );
        orderbooks.insert(
            "secondary".to_string(),
            OrderbookCfg {
                document: default_document(),
                key: "secondary".to_string(),
                address: Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
                network,
                subgraph,
                local_db_remote: None,
                label: None,
                deployment_block: 67890,
            },
        );

        let yaml = OrderbookCfg::to_yaml_hash(&orderbooks).unwrap();

        let StrictYaml::Hash(orderbooks_hash) = yaml else {
            panic!("orderbooks were not serialized to a YAML hash");
        };
        let Some(StrictYaml::Hash(primary_hash)) =
            orderbooks_hash.get(&StrictYaml::String("primary".to_string()))
        else {
            panic!("primary orderbook missing from serialized YAML");
        };
        let Some(StrictYaml::Hash(secondary_hash)) =
            orderbooks_hash.get(&StrictYaml::String("secondary".to_string()))
        else {
            panic!("secondary orderbook missing from serialized YAML");
        };

        assert_eq!(
            primary_hash.get(&StrictYaml::String("address".to_string())),
            Some(&StrictYaml::String(
                "0x0000000000000000000000000000000000000001".to_string()
            ))
        );
        assert_eq!(
            primary_hash.get(&StrictYaml::String("network".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            primary_hash.get(&StrictYaml::String("subgraph".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            primary_hash.get(&StrictYaml::String("local-db-remote".to_string())),
            Some(&StrictYaml::String("mainnet-remote".to_string()))
        );
        assert_eq!(
            primary_hash.get(&StrictYaml::String("label".to_string())),
            Some(&StrictYaml::String("Primary Orderbook".to_string()))
        );
        assert_eq!(
            primary_hash.get(&StrictYaml::String("deployment-block".to_string())),
            Some(&StrictYaml::String("12345".to_string()))
        );

        assert_eq!(
            secondary_hash.get(&StrictYaml::String("address".to_string())),
            Some(&StrictYaml::String(
                "0x0000000000000000000000000000000000000002".to_string()
            ))
        );
        assert_eq!(
            secondary_hash.get(&StrictYaml::String("network".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            secondary_hash.get(&StrictYaml::String("subgraph".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            secondary_hash.get(&StrictYaml::String("deployment-block".to_string())),
            Some(&StrictYaml::String("67890".to_string()))
        );
        assert!(!secondary_hash.contains_key(&StrictYaml::String("label".to_string())));
        assert!(!secondary_hash.contains_key(&StrictYaml::String("local-db-remote".to_string())));
    }
}
