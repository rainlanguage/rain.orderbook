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

const ALLOWED_REGISTRY_KEYS: [&str; 2] = ["address", "network"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct RegistryCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub network: Arc<NetworkCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RegistryCfg);

impl RegistryCfg {
    pub fn dummy() -> Self {
        RegistryCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: Arc::new(NetworkCfg::dummy()),
        }
    }

    pub fn validate_address(value: &str) -> Result<Address, ParseRegistryConfigSourceError> {
        Address::from_str(value).map_err(ParseRegistryConfigSourceError::AddressParseError)
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        registry_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(registries_hash) = require_hash(&document_read, Some("registries"), None) {
                if let Some(registry_yaml) =
                    registries_hash.get(&StrictYaml::String(registry_key.to_string()))
                {
                    return require_string(registry_yaml, Some("network"), None)
                        .or_else(|_| Ok(registry_key.to_string()));
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "registries".to_string(),
                        expected: "a map".to_string(),
                    },
                    location: "root".to_string(),
                });
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("network for registry '{}'", registry_key)),
            location: "root".to_string(),
        })
    }
}

impl Default for RegistryCfg {
    fn default() -> Self {
        RegistryCfg::dummy()
    }
}

impl PartialEq for RegistryCfg {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.network == other.network
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseRegistryConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(alloy::primitives::hex::FromHexError),
    #[error("Network not found for Registry: {0}")]
    NetworkNotFoundError(String),
}

impl ParseRegistryConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseRegistryConfigSourceError::AddressParseError(err) =>
                format!("The registry address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseRegistryConfigSourceError::NetworkNotFoundError(network) =>
                format!("The network '{}' specified for this registry was not found in your YAML configuration. Please define this network or use an existing one.", network),
        }
    }
}

impl YamlParsableHash for RegistryCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut registries = HashMap::new();

        let networks = NetworkCfg::parse_all_from_yaml(documents.clone(), context)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(registries_hash) = require_hash(&document_read, Some("registries"), None) {
                for (key_yaml, registry_yaml) in registries_hash {
                    let registry_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("registry '{}'", registry_key);

                    let address_str =
                        require_string(registry_yaml, Some("address"), Some(location.clone()))?;
                    let address = RegistryCfg::validate_address(&address_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let network_name = match optional_string(registry_yaml, "network") {
                        Some(network_name) => network_name,
                        None => registry_key.clone(),
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

                    let registry = RegistryCfg {
                        document: document.clone(),
                        key: registry_key.clone(),
                        address,
                        network: Arc::new(network.clone()),
                    };

                    if registries.contains_key(&registry_key) {
                        return Err(YamlError::KeyShadowing(
                            registry_key,
                            "registries".to_string(),
                        ));
                    }
                    registries.insert(registry_key, registry);
                }
            }
        }

        if registries.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("registries".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(registries)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let registries_key = StrictYaml::String("registries".to_string());
            let Some(registries_value) = root_hash.get(&registries_key) else {
                continue;
            };
            let StrictYaml::Hash(ref registries_hash) = registries_value.clone() else {
                continue;
            };

            let mut sanitized_registries: Vec<(String, StrictYaml)> = Vec::new();

            for (key, value) in registries_hash {
                let Some(key_str) = key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref registry_hash) = *value else {
                    continue;
                };

                let mut sanitized = Hash::new();
                for allowed_key in ALLOWED_REGISTRY_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = registry_hash.get(&key_yaml) {
                        sanitized.insert(key_yaml, v.clone());
                    }
                }
                sanitized_registries.push((key_str.to_string(), StrictYaml::Hash(sanitized)));
            }

            sanitized_registries.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_registries_hash = Hash::new();
            for (key, value) in sanitized_registries {
                new_registries_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(registries_key, StrictYaml::Hash(new_registries_hash));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_registries_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
registries:
    RegistryOne:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let yaml_two = r#"
registries:
    RegistryTwo:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let registries = RegistryCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(registries.len(), 2);
        assert!(registries.contains_key("RegistryOne"));
        assert!(registries.contains_key("RegistryTwo"));

        assert_eq!(
            registries.get("RegistryOne").unwrap().address.to_string(),
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(
            registries.get("RegistryTwo").unwrap().address.to_string(),
            "0x0987654321098765432109876543210987654321"
        );
    }

    #[test]
    fn test_parse_registries_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
registries:
    DuplicateRegistry:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let yaml_two = r#"
registries:
    DuplicateRegistry:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = RegistryCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateRegistry".to_string(), "registries".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'DuplicateRegistry' is defined multiple times in your YAML configuration at registries"
        );
    }

    #[test]
    fn test_parse_registry_from_yaml_network_key() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
registries:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
"#;

        let documents = vec![get_document(yaml)];
        let network_key = RegistryCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
registries:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
"#;
        let documents = vec![get_document(yaml)];
        let network_key = RegistryCfg::parse_network_key(documents, "mainnet").unwrap();
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
registries: test
"#;
        let error =
            RegistryCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "registries".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'registries' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
registries:
  - test
"#;
        let error =
            RegistryCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "registries".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'registries' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
registries:
  - test: test
"#;
        let error =
            RegistryCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "registries".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'registries' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
registries:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
"#;
        let res = RegistryCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap();
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
registries:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let document = get_document(yaml);
        RegistryCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let registries = root
            .get(&StrictYaml::String("registries".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref registries_hash) = *registries else {
            panic!("expected registries hash");
        };
        let mainnet = registries_hash
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
registries:
    mainnet:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
        extra: dropped
"#;
        let document = get_document(yaml);
        RegistryCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let registries = root
            .get(&StrictYaml::String("registries".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref registries_hash) = *registries else {
            panic!("expected registries hash");
        };
        let mainnet = registries_hash
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
registries:
    mainnet: not-a-hash
"#;
        let document = get_document(yaml);
        RegistryCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let registries = root
            .get(&StrictYaml::String("registries".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref registries_hash) = *registries else {
            panic!("expected registries hash");
        };

        assert!(!registries_hash.contains_key(&StrictYaml::String("mainnet".to_string())));
        assert!(registries_hash.is_empty());
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
registries:
    zebra:
        address: 0x0000000000000000000000000000000000000003
    alpha:
        address: 0x0000000000000000000000000000000000000001
    beta:
        address: 0x0000000000000000000000000000000000000002
"#;
        let document = get_document(yaml);
        RegistryCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let registries = root
            .get(&StrictYaml::String("registries".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref registries_hash) = *registries else {
            panic!("expected registries hash");
        };

        let keys: Vec<String> = registries_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_handles_missing_registries_section() {
        let yaml = r#"
other: value
"#;
        let document = get_document(yaml);
        RegistryCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        assert!(!root.contains_key(&StrictYaml::String("registries".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = r#"just a string"#;
        let document = get_document(yaml);
        RegistryCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_registries() {
        let yaml = r#"
registries: not-a-hash
"#;
        let document = get_document(yaml);
        RegistryCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let registries = root
            .get(&StrictYaml::String("registries".to_string()))
            .unwrap();
        assert_eq!(registries.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_documents_per_doc_no_cross_merge() {
        let yaml_one = r#"
registries:
    registry-one:
        address: 0x0000000000000000000000000000000000000001
        extra-key: dropped
"#;
        let yaml_two = r#"
registries:
    registry-two:
        address: 0x0000000000000000000000000000000000000002
        another-extra: also-dropped
"#;
        let doc_one = get_document(yaml_one);
        let doc_two = get_document(yaml_two);
        let documents = vec![doc_one.clone(), doc_two.clone()];
        RegistryCfg::sanitize_documents(&documents).unwrap();

        {
            let doc_read = doc_one.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let registries = root
                .get(&StrictYaml::String("registries".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref registries_hash) = *registries else {
                panic!("expected registries hash");
            };

            let keys: Vec<String> = registries_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["registry-one"]);

            let registry = registries_hash
                .get(&StrictYaml::String("registry-one".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref registry_hash) = *registry else {
                panic!("expected registry hash");
            };
            assert!(!registry_hash.contains_key(&StrictYaml::String("extra-key".to_string())));
        }

        {
            let doc_read = doc_two.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let registries = root
                .get(&StrictYaml::String("registries".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref registries_hash) = *registries else {
                panic!("expected registries hash");
            };

            let keys: Vec<String> = registries_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["registry-two"]);

            let registry = registries_hash
                .get(&StrictYaml::String("registry-two".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref registry_hash) = *registry else {
                panic!("expected registry hash");
            };
            assert!(!registry_hash.contains_key(&StrictYaml::String("another-extra".to_string())));
        }
    }
}
