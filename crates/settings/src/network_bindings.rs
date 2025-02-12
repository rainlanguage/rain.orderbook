use crate::{
    yaml::{default_document, optional_hash, FieldErrorKind, YamlError, YamlParsableHash},
    Network,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkBinding {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub bindings: HashMap<String, String>,
}

impl YamlParsableHash for NetworkBinding {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _context: Option<&crate::yaml::context::Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut network_bindings = HashMap::new();

        let network_keys = Network::parse_network_keys(documents.clone())?;

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(bindings_yaml) = optional_hash(&document_read, "network-bindings") {
                for (network_key, binding_yaml) in bindings_yaml {
                    let network_key = network_key.as_str().unwrap_or_default().to_string();
                    let location = format!("network-bindings '{network_key}'");

                    if !network_keys.contains(&network_key) {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "networks".to_string(),
                                reason: format!("Network '{}' not found", network_key),
                            },
                            location: location.clone(),
                        });
                    }

                    let bindings_map = binding_yaml
                        .as_hash()
                        .ok_or(YamlError::Field {
                            kind: FieldErrorKind::InvalidType {
                                field: network_key.clone(),
                                expected: "a map".to_string(),
                            },
                            location: location.clone(),
                        })?
                        .iter()
                        .map(|(k, v)| {
                            Ok((
                                k.as_str()
                                    .ok_or(YamlError::Field {
                                        kind: FieldErrorKind::InvalidType {
                                            field: "binding key".to_string(),
                                            expected: "a string".to_string(),
                                        },
                                        location: location.clone(),
                                    })?
                                    .to_string(),
                                v.as_str()
                                    .ok_or(YamlError::Field {
                                        kind: FieldErrorKind::InvalidType {
                                            field: "binding value".to_string(),
                                            expected: "a string".to_string(),
                                        },
                                        location: location.clone(),
                                    })?
                                    .to_string(),
                            ))
                        })
                        .collect::<Result<HashMap<_, _>, YamlError>>()?;

                    let network_binding = NetworkBinding {
                        document: document.clone(),
                        key: network_key.clone(),
                        bindings: bindings_map,
                    };

                    if network_bindings.contains_key(&network_key) {
                        return Err(YamlError::KeyShadowing(network_key));
                    }
                    network_bindings.insert(network_key, network_binding);
                }
            }
        }

        Ok(network_bindings)
    }
}

impl Default for NetworkBinding {
    fn default() -> Self {
        Self {
            document: default_document(),
            key: "".to_string(),
            bindings: HashMap::new(),
        }
    }
}

impl PartialEq for NetworkBinding {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.bindings == other.bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_missing_network() {
        let yaml = r#"
network-bindings:
    network1:
        binding1: value1
"#;
        let error =
            NetworkBinding::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "networks".to_string(),
                    reason: "Network 'network1' not found".to_string()
                },
                location: "network-bindings 'network1'".to_string()
            }
        );
    }

    #[test]
    fn test_network_bindings_invalid_yaml() {
        let yaml = r#"
networks:
    network1:
        rpc: https://mainnet.infura.io
        chain-id: 1
network-bindings:
    network1:
        - invalid
"#;
        let error =
            NetworkBinding::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "network1".to_string(),
                    expected: "a map".to_string(),
                },
                location: "network-bindings 'network1'".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://mainnet.infura.io
        chain-id: 1
network-bindings:
    network1:
        - invalid: invalid
"#;
        let error =
            NetworkBinding::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "network1".to_string(),
                    expected: "a map".to_string(),
                },
                location: "network-bindings 'network1'".to_string(),
            }
        );
    }

    #[test]
    fn test_network_bindings_duplicate_key() {
        let yaml1 = r#"
networks:
    network1:
        rpc: https://mainnet.infura.io
        chain-id: 1
network-bindings:
    network1:
        binding1: value1
"#;
        let yaml2 = r#"
network-bindings:
    network1:
        binding2: value2
"#;
        let error = NetworkBinding::parse_all_from_yaml(
            vec![get_document(yaml1), get_document(yaml2)],
            None,
        )
        .unwrap_err();
        assert_eq!(error, YamlError::KeyShadowing("network1".to_string()));
    }
}
