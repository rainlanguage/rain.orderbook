use crate::config_source::*;
use crate::yaml::context::Context;
use crate::yaml::{
    default_document, optional_string, require_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    num::ParseIntError,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use typeshare::typeshare;
use url::{ParseError, Url};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub rpc: Url,
    #[typeshare(typescript(type = "number"))]
    pub chain_id: u64,
    pub label: Option<String>,
    #[typeshare(typescript(type = "number"))]
    pub network_id: Option<u64>,
    pub currency: Option<String>,
}
impl Network {
    pub fn dummy() -> Self {
        Network {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            rpc: Url::parse("http://rpc.com").unwrap(),
            chain_id: 1,
            label: None,
            network_id: None,
            currency: None,
        }
    }

    pub fn validate_rpc(value: &str) -> Result<Url, ParseNetworkConfigSourceError> {
        Url::parse(value).map_err(ParseNetworkConfigSourceError::RpcParseError)
    }
    pub fn validate_chain_id(value: &str) -> Result<u64, ParseNetworkConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseNetworkConfigSourceError::ChainIdParseError)
    }
    pub fn validate_network_id(value: &str) -> Result<u64, ParseNetworkConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseNetworkConfigSourceError::NetworkIdParseError)
    }

    pub fn update_rpc(&mut self, rpc: &str) -> Result<Self, YamlError> {
        let rpc = Network::validate_rpc(rpc)?;

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut networks)) =
                document_hash.get_mut(&StrictYaml::String("networks".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut network)) =
                    networks.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    network[&StrictYaml::String("rpc".to_string())] =
                        StrictYaml::String(rpc.to_string());
                    self.rpc = rpc;
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "networks".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("networks".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(self.clone())
    }

    pub fn parse_rpc(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        network_key: &str,
    ) -> Result<Url, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            let networks_hash =
                require_hash(&document_read, Some("networks"), Some("root".to_string()))?;

            if let Some(network_yaml) =
                networks_hash.get(&StrictYaml::String(network_key.to_string()))
            {
                let location = format!("network '{}'", network_key);
                let rpc_str = require_string(network_yaml, Some("rpc"), Some(location.clone()))?;

                return Network::validate_rpc(&rpc_str).map_err(|e| YamlError::Field {
                    kind: FieldErrorKind::InvalidValue {
                        field: "rpc".to_string(),
                        reason: e.to_string(),
                    },
                    location,
                });
            }
        }

        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("rpc for network '{}'", network_key)),
            location: "root".to_string(),
        })
    }

    pub fn parse_network_keys(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<Vec<String>, YamlError> {
        let mut networks = Vec::new();
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(networks_hash) =
                require_hash(&document_read, Some("networks"), Some("root".to_string()))
            {
                for (key_yaml, _) in networks_hash {
                    let network_key = key_yaml.as_str().unwrap_or_default().to_string();
                    networks.push(network_key);
                }
            }
        }

        Ok(networks)
    }
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Network);

impl YamlParsableHash for Network {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut networks = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(networks_hash) =
                require_hash(&document_read, Some("networks"), Some("root".to_string()))
            {
                for (key_yaml, network_yaml) in networks_hash {
                    let network_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("network '{}'", network_key);

                    let rpc_str =
                        require_string(network_yaml, Some("rpc"), Some(location.clone()))?;

                    let rpc_url =
                        Network::validate_rpc(&rpc_str).map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "rpc".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let chain_id_str =
                        require_string(network_yaml, Some("chain-id"), Some(location.clone()))?;

                    let chain_id = chain_id_str.parse::<u64>().map_err(|e| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "chain-id".to_string(),
                            reason: e.to_string(),
                        },
                        location: location.clone(),
                    })?;

                    let label = optional_string(network_yaml, "label");
                    let network_id = optional_string(network_yaml, "network-id")
                        .map(|id| Network::validate_network_id(&id))
                        .transpose()
                        .map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "network-id".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let currency = optional_string(network_yaml, "currency");

                    let network = Network {
                        document: document.clone(),
                        key: network_key.clone(),
                        rpc: rpc_url,
                        chain_id,
                        label,
                        network_id,
                        currency,
                    };

                    if networks.contains_key(&network_key) {
                        return Err(YamlError::KeyShadowing(network_key));
                    }
                    networks.insert(network_key, network);
                }
            }
        }

        if networks.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(networks)
    }
}

impl Default for Network {
    fn default() -> Self {
        Network::dummy()
    }
}
impl PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.rpc == other.rpc
            && self.chain_id == other.chain_id
            && self.label == other.label
            && self.network_id == other.network_id
            && self.currency == other.currency
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseNetworkConfigSourceError {
    #[error("Failed to parse rpc: {}", 0)]
    RpcParseError(ParseError),
    #[error("Failed to parse chain_id: {}", 0)]
    ChainIdParseError(ParseIntError),
    #[error("Failed to parse network_id: {}", 0)]
    NetworkIdParseError(ParseIntError),
}

impl NetworkConfigSource {
    pub fn try_into_network(self, key: String) -> Result<Network, ParseNetworkConfigSourceError> {
        Ok(Network {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key,
            rpc: self.rpc,
            chain_id: self.chain_id,
            label: self.label,
            network_id: self.network_id,
            currency: self.currency,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;
    use url::Url;

    #[test]
    fn test_try_from_network_string_success() {
        let network_string = NetworkConfigSource {
            rpc: Url::parse("http://127.0.0.1:8545").unwrap(),
            chain_id: 1,
            network_id: Some(1),
            label: Some("Local Testnet".into()),
            currency: Some("ETH".into()),
        };

        let result = network_string.try_into_network("local".into());
        assert!(result.is_ok());
        let network = result.unwrap();

        assert_eq!(network.rpc, Url::parse("http://127.0.0.1:8545").unwrap());
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.label, Some("Local Testnet".into()));
        assert_eq!(network.currency, Some("ETH".into()));
        assert_eq!(network.key, "local");
    }

    #[test]
    fn test_parse_networks_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Network::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            }
        );

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = Network::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("rpc".to_string()),
                location: "network 'mainnet'".to_string(),
            }
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
"#;
        let error = Network::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("chain-id".to_string()),
                location: "network 'mainnet'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_networks_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
    testnet:
        rpc: https://testnet.infura.io
        chain-id: 2
"#;
        let yaml_two = r#"
networks:
    network-one:
        rpc: https://network-one.infura.io
        chain-id: 3
    network-two:
        rpc: https://network-two.infura.io
        chain-id: 4
"#;
        let networks = Network::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        assert_eq!(networks.len(), 4);
        assert_eq!(
            networks.get("mainnet").unwrap().rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );
        assert_eq!(
            networks.get("testnet").unwrap().rpc,
            Url::parse("https://testnet.infura.io").unwrap()
        );
        assert_eq!(
            networks.get("network-one").unwrap().rpc,
            Url::parse("https://network-one.infura.io").unwrap()
        );
        assert_eq!(
            networks.get("network-two").unwrap().rpc,
            Url::parse("https://network-two.infura.io").unwrap()
        );
    }

    #[test]
    fn test_parse_networks_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
    testnet:
        rpc: https://mainnet.infura.io
        chain-id: 2
"#;
        let yaml_two = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
"#;
        let error = Network::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(error, YamlError::KeyShadowing("mainnet".to_string()));
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
networks: test
"#;
        let error = Network::parse_rpc(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "networks".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );

        let yaml = r#"
networks:
  - test
"#;
        let error = Network::parse_rpc(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "networks".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );

        let yaml = r#"
networks:
  - test: test
"#;
        let error = Network::parse_rpc(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "networks".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );

        let yaml = r#"
networks:
  mainnet:
    rpc: https://rpc.com
    chain-id: 1
"#;
        let res = Network::parse_rpc(vec![get_document(yaml)], "mainnet").unwrap();
        assert_eq!(res, Url::parse("https://rpc.com").unwrap());
    }

    #[test]
    fn test_parse_network_keys() {
        let yaml = r#"
networks:
  mainnet:
    rpc: https://mainnet.infura.io
    chain-id: 1
  testnet:
    rpc: https://testnet.infura.io
    chain-id: 2
"#;
        let networks = Network::parse_network_keys(vec![get_document(yaml)]).unwrap();
        assert_eq!(networks, vec!["mainnet", "testnet"]);
    }
}
