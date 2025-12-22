use crate::*;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::{
    context::Context, default_document, optional_string, require_hash, require_string,
    FieldErrorKind, YamlError, YamlParsableHash,
};

const ALLOWED_DEPLOYER_KEYS: [&str; 2] = ["address", "network"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct DeployerCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub network: Arc<NetworkCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeployerCfg);

impl DeployerCfg {
    pub fn dummy() -> Self {
        DeployerCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: Arc::new(NetworkCfg::dummy()),
        }
    }

    pub fn validate_address(value: &str) -> Result<Address, ParseDeployerConfigSourceError> {
        Address::from_str(value).map_err(ParseDeployerConfigSourceError::AddressParseError)
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployer_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(deployers_hash) = require_hash(&document_read, Some("deployers"), None) {
                if let Some(deployer_yaml) =
                    deployers_hash.get(&StrictYaml::String(deployer_key.to_string()))
                {
                    return require_string(deployer_yaml, Some("network"), None)
                        .or_else(|_| Ok(deployer_key.to_string()));
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "deployers".to_string(),
                        expected: "a map".to_string(),
                    },
                    location: "root".to_string(),
                });
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("network for deployer '{}'", deployer_key)),
            location: "root".to_string(),
        })
    }
}

impl Default for DeployerCfg {
    fn default() -> Self {
        DeployerCfg::dummy()
    }
}

impl PartialEq for DeployerCfg {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.network == other.network
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseDeployerConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(alloy::primitives::hex::FromHexError),
    #[error("Network not found for Deployer: {0}")]
    NetworkNotFoundError(String),
}

impl ParseDeployerConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseDeployerConfigSourceError::AddressParseError(err) =>
                format!("The deployer address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseDeployerConfigSourceError::NetworkNotFoundError(network) =>
                format!("The network '{}' specified for this deployer was not found in your YAML configuration. Please define this network or use an existing one.", network),
        }
    }
}

impl YamlParsableHash for DeployerCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut deployers = HashMap::new();

        let networks = NetworkCfg::parse_all_from_yaml(documents.clone(), context)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(deployers_hash) = require_hash(&document_read, Some("deployers"), None) {
                for (key_yaml, deployer_yaml) in deployers_hash {
                    let deployer_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("deployer '{}'", deployer_key);

                    let address_str =
                        require_string(deployer_yaml, Some("address"), Some(location.clone()))?;
                    let address = DeployerCfg::validate_address(&address_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let network_name = match optional_string(deployer_yaml, "network") {
                        Some(network_name) => network_name,
                        None => deployer_key.clone(),
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

                    let deployer = DeployerCfg {
                        document: document.clone(),
                        key: deployer_key.clone(),
                        address,
                        network: Arc::new(network.clone()),
                    };

                    if deployers.contains_key(&deployer_key) {
                        return Err(YamlError::KeyShadowing(
                            deployer_key,
                            "deployers".to_string(),
                        ));
                    }
                    deployers.insert(deployer_key, deployer);
                }
            }
        }

        if deployers.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("deployers".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(deployers)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let deployers_key = StrictYaml::String("deployers".to_string());
            let Some(deployers_value) = root_hash.get(&deployers_key) else {
                continue;
            };
            let StrictYaml::Hash(ref deployers_hash) = deployers_value.clone() else {
                continue;
            };

            let mut sanitized_deployers: Vec<(String, StrictYaml)> = Vec::new();

            for (key, value) in deployers_hash {
                let Some(key_str) = key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref deployer_hash) = *value else {
                    continue;
                };

                let mut sanitized = Hash::new();
                for allowed_key in ALLOWED_DEPLOYER_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = deployer_hash.get(&key_yaml) {
                        sanitized.insert(key_yaml, v.clone());
                    }
                }
                sanitized_deployers.push((key_str.to_string(), StrictYaml::Hash(sanitized)));
            }

            sanitized_deployers.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_deployers_hash = Hash::new();
            for (key, value) in sanitized_deployers {
                new_deployers_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(deployers_key, StrictYaml::Hash(new_deployers_hash));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_deployers_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
deployers:
    DeployerOne:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let yaml_two = r#"
deployers:
    DeployerTwo:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let deployers = DeployerCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(deployers.len(), 2);
        assert!(deployers.contains_key("DeployerOne"));
        assert!(deployers.contains_key("DeployerTwo"));

        assert_eq!(
            deployers.get("DeployerOne").unwrap().address.to_string(),
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(
            deployers.get("DeployerTwo").unwrap().address.to_string(),
            "0x0987654321098765432109876543210987654321"
        );
    }

    #[test]
    fn test_parse_deployers_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
deployers:
    DuplicateDeployer:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let yaml_two = r#"
deployers:
    DuplicateDeployer:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = DeployerCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateDeployer".to_string(), "deployers".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'DuplicateDeployer' is defined multiple times in your YAML configuration at deployers"
        );
    }

    #[test]
    fn test_parse_deployer_from_yaml_network_key() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
deployers:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
"#;

        let documents = vec![get_document(yaml)];
        let network_key = DeployerCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
deployers:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
"#;
        let documents = vec![get_document(yaml)];
        let network_key = DeployerCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
deployers: test
"#;
        let error =
            DeployerCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployers".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'deployers' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
deployers:
  - test
"#;
        let error =
            DeployerCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployers".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'deployers' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
deployers:
  - test: test
"#;
        let error =
            DeployerCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployers".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'deployers' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
deployers:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
"#;
        let res = DeployerCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap();
        assert_eq!(res, "mainnet");
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_keys() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
deployers:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let document = get_document(yaml);
        DeployerCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployers = root
            .get(&StrictYaml::String("deployers".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployers_hash) = *deployers else {
            panic!("expected deployers hash");
        };
        let mainnet = deployers_hash
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref mainnet_hash) = *mainnet else {
            panic!("expected mainnet hash");
        };

        assert!(mainnet_hash.contains_key(&StrictYaml::String("address".to_string())));
        assert!(mainnet_hash.contains_key(&StrictYaml::String("network".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("another-unknown".to_string())));
        assert_eq!(mainnet_hash.len(), 2);
    }

    #[test]
    fn test_sanitize_documents_preserves_allowed_key_order() {
        let yaml = r#"
deployers:
    mainnet:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
        extra: dropped
"#;
        let document = get_document(yaml);
        DeployerCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployers = root
            .get(&StrictYaml::String("deployers".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployers_hash) = *deployers else {
            panic!("expected deployers hash");
        };
        let mainnet = deployers_hash
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref mainnet_hash) = *mainnet else {
            panic!("expected mainnet hash");
        };

        let keys: Vec<String> = mainnet_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["address", "network"]);
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_entries() {
        let yaml = r#"
deployers:
    mainnet: not-a-hash
"#;
        let document = get_document(yaml);
        DeployerCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployers = root
            .get(&StrictYaml::String("deployers".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployers_hash) = *deployers else {
            panic!("expected deployers hash");
        };

        assert!(!deployers_hash.contains_key(&StrictYaml::String("mainnet".to_string())));
        assert!(deployers_hash.is_empty());
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
deployers:
    zebra:
        address: 0x0000000000000000000000000000000000000003
    alpha:
        address: 0x0000000000000000000000000000000000000001
    beta:
        address: 0x0000000000000000000000000000000000000002
"#;
        let document = get_document(yaml);
        DeployerCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployers = root
            .get(&StrictYaml::String("deployers".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployers_hash) = *deployers else {
            panic!("expected deployers hash");
        };

        let keys: Vec<String> = deployers_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_handles_missing_deployers_section() {
        let yaml = r#"
other: value
"#;
        let document = get_document(yaml);
        DeployerCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        assert!(!root.contains_key(&StrictYaml::String("deployers".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = r#"just a string"#;
        let document = get_document(yaml);
        DeployerCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_deployers() {
        let yaml = r#"
deployers: not-a-hash
"#;
        let document = get_document(yaml);
        DeployerCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployers = root
            .get(&StrictYaml::String("deployers".to_string()))
            .unwrap();
        assert_eq!(deployers.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_documents_per_doc_no_cross_merge() {
        let yaml_one = r#"
deployers:
    deployer-one:
        address: 0x0000000000000000000000000000000000000001
        extra-key: dropped
"#;
        let yaml_two = r#"
deployers:
    deployer-two:
        address: 0x0000000000000000000000000000000000000002
        another-extra: also-dropped
"#;
        let doc_one = get_document(yaml_one);
        let doc_two = get_document(yaml_two);
        let documents = vec![doc_one.clone(), doc_two.clone()];
        DeployerCfg::sanitize_documents(&documents).unwrap();

        {
            let doc_read = doc_one.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let deployers = root
                .get(&StrictYaml::String("deployers".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployers_hash) = *deployers else {
                panic!("expected deployers hash");
            };

            let keys: Vec<String> = deployers_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["deployer-one"]);

            let deployer = deployers_hash
                .get(&StrictYaml::String("deployer-one".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployer_hash) = *deployer else {
                panic!("expected deployer hash");
            };
            assert!(!deployer_hash.contains_key(&StrictYaml::String("extra-key".to_string())));
        }

        {
            let doc_read = doc_two.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let deployers = root
                .get(&StrictYaml::String("deployers".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployers_hash) = *deployers else {
                panic!("expected deployers hash");
            };

            let keys: Vec<String> = deployers_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["deployer-two"]);

            let deployer = deployers_hash
                .get(&StrictYaml::String("deployer-two".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployer_hash) = *deployer else {
                panic!("expected deployer hash");
            };
            assert!(!deployer_hash.contains_key(&StrictYaml::String("another-extra".to_string())));
        }
    }
}
