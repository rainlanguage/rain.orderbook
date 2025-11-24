use crate::yaml::context::Context;
use crate::yaml::{
    default_document, optional_string, require_hash, require_string, require_vec, FieldErrorKind,
    YamlError, YamlParsableHash,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    num::ParseIntError,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct NetworkCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string[]"))]
    pub rpcs: Vec<Url>,
    pub chain_id: u32,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub network_id: Option<u32>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub currency: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(NetworkCfg);

impl NetworkCfg {
    pub fn dummy() -> Self {
        NetworkCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            rpcs: vec![Url::parse("http://rpc.com").unwrap()],
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
    pub fn validate_network_id(value: &str) -> Result<u32, ParseNetworkConfigSourceError> {
        value
            .parse::<u32>()
            .map_err(ParseNetworkConfigSourceError::NetworkIdParseError)
    }

    pub fn update_rpcs(&mut self, rpcs: Vec<String>) -> Result<Self, YamlError> {
        let mut rpc_vec = Vec::new();
        for rpc in rpcs {
            rpc_vec.push(NetworkCfg::validate_rpc(&rpc)?);
        }

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
                    let rpcs = rpc_vec
                        .iter()
                        .map(|rpc| StrictYaml::String(rpc.to_string()))
                        .collect();
                    network[&StrictYaml::String("rpcs".to_string())] = StrictYaml::Array(rpcs);
                    self.rpcs = rpc_vec;
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

    pub fn parse_rpcs(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        network_key: &str,
    ) -> Result<Vec<Url>, YamlError> {
        let mut res = Vec::new();

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            let networks_hash =
                require_hash(&document_read, Some("networks"), Some("root".to_string()))?;

            if let Some(network_yaml) =
                networks_hash.get(&StrictYaml::String(network_key.to_string()))
            {
                let location = format!("network '{}'", network_key);
                let rpcs = require_vec(network_yaml, "rpcs", Some(location.clone()))?;

                for rpc_value in rpcs {
                    let url_str = require_string(rpc_value, None, None)?;
                    let url = NetworkCfg::validate_rpc(&url_str)?;
                    res.push(url);
                }

                if res.is_empty() {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "rpcs".to_string(),
                            reason: "must be a non-empty array".to_string(),
                        },
                        location,
                    });
                }

                return Ok(res);
            }
        }

        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("rpcs for network '{}'", network_key)),
            location: "root".to_string(),
        })
    }
}

impl YamlParsableHash for NetworkCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
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

                    let mut rpcs = Vec::new();
                    let rpc_vec = require_vec(network_yaml, "rpcs", Some(location.clone()))?;

                    for rpc_value in rpc_vec {
                        let rpc_str = require_string(rpc_value, None, None)?;
                        let rpc_url = NetworkCfg::validate_rpc(&rpc_str)?;
                        rpcs.push(rpc_url);
                    }
                    if rpcs.is_empty() {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "rpcs".to_string(),
                                reason: "must be a non-empty array".to_string(),
                            },
                            location: location.clone(),
                        });
                    }

                    let chain_id_str =
                        require_string(network_yaml, Some("chain-id"), Some(location.clone()))?;

                    let chain_id = chain_id_str.parse::<u32>().map_err(|e| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "chain-id".to_string(),
                            reason: e.to_string(),
                        },
                        location: location.clone(),
                    })?;

                    let label = optional_string(network_yaml, "label");
                    let network_id = optional_string(network_yaml, "network-id")
                        .map(|id| NetworkCfg::validate_network_id(&id))
                        .transpose()
                        .map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "network-id".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let currency = optional_string(network_yaml, "currency");

                    let network = NetworkCfg {
                        document: document.clone(),
                        key: network_key.clone(),
                        rpcs,
                        chain_id,
                        label,
                        network_id,
                        currency,
                    };

                    if networks.contains_key(&network_key) {
                        return Err(YamlError::KeyShadowing(network_key, "networks".to_string()));
                    }
                    networks.insert(network_key, network);
                }
            }
        }

        if let Some(context) = context {
            if let Some(yaml_cache) = &context.yaml_cache {
                for (key, network) in &yaml_cache.remote_networks {
                    if networks.contains_key(key) {
                        return Err(YamlError::ParseNetworkConfigSourceError(
                            ParseNetworkConfigSourceError::RemoteNetworkKeyShadowing(key.clone()),
                        ));
                    }
                    networks.insert(key.clone(), network.clone());
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

    fn to_yaml_hash(networks: &HashMap<String, Self>) -> Result<StrictYaml, YamlError> {
        let mut networks_yaml = Hash::new();

        for (key, cfg) in networks {
            let mut network_yaml = Hash::new();

            let rpcs = cfg
                .rpcs
                .iter()
                .map(|rpc| StrictYaml::String(rpc.to_string()))
                .collect();
            network_yaml.insert(
                StrictYaml::String("rpcs".to_string()),
                StrictYaml::Array(rpcs),
            );

            network_yaml.insert(
                StrictYaml::String("chain-id".to_string()),
                StrictYaml::String(cfg.chain_id.to_string()),
            );

            if let Some(label) = &cfg.label {
                network_yaml.insert(
                    StrictYaml::String("label".to_string()),
                    StrictYaml::String(label.clone()),
                );
            }

            if let Some(network_id) = cfg.network_id {
                network_yaml.insert(
                    StrictYaml::String("network-id".to_string()),
                    StrictYaml::String(network_id.to_string()),
                );
            }

            if let Some(currency) = &cfg.currency {
                network_yaml.insert(
                    StrictYaml::String("currency".to_string()),
                    StrictYaml::String(currency.clone()),
                );
            }

            networks_yaml.insert(
                StrictYaml::String(key.clone()),
                StrictYaml::Hash(network_yaml),
            );
        }

        Ok(StrictYaml::Hash(networks_yaml))
    }
}

impl Default for NetworkCfg {
    fn default() -> Self {
        NetworkCfg::dummy()
    }
}
impl PartialEq for NetworkCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.rpcs == other.rpcs
            && self.chain_id == other.chain_id
            && self.label == other.label
            && self.network_id == other.network_id
            && self.currency == other.currency
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseNetworkConfigSourceError {
    #[error("Failed to parse rpcs: {0}")]
    RpcParseError(ParseError),
    #[error("Failed to parse chain_id: {0}")]
    ChainIdParseError(ParseIntError),
    #[error("Failed to parse network_id: {0}")]
    NetworkIdParseError(ParseIntError),
    #[error("Remote network key shadowing: {0}")]
    RemoteNetworkKeyShadowing(String),
}

impl ParseNetworkConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseNetworkConfigSourceError::RpcParseError(err) => format!(
                "The RPC URL in your network configuration is invalid: {}",
                err
            ),
            ParseNetworkConfigSourceError::ChainIdParseError(err) => format!(
                "The chain ID in your network configuration must be a valid number: {}",
                err
            ),
            ParseNetworkConfigSourceError::NetworkIdParseError(err) => format!(
                "The network ID in your network configuration must be a valid number: {}",
                err
            ),
            ParseNetworkConfigSourceError::RemoteNetworkKeyShadowing(key) => format!(
                "The remote network key '{}' is already defined in network configuration",
                key
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;
    use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
    use url::Url;

    #[test]
    fn test_parse_networks_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
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
    mainnet:
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("rpcs".to_string()),
                location: "network 'mainnet'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'rpcs' in network 'mainnet'"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "rpcs".to_string(),
                    expected: "a vector".to_string(),
                },
                location: "network 'mainnet'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'rpcs' in network 'mainnet' must be a vector"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            -
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseNetworkConfigSourceError(ParseNetworkConfigSourceError::RpcParseError(
                ParseError::RelativeUrlWithoutBase
            ))
        );
        assert_eq!(
            error.to_readable_msg(),
            "Network configuration error in your YAML: Failed to parse rpcs: relative URL without a base"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("chain-id".to_string()),
                location: "network 'mainnet'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'chain-id' in network 'mainnet'"
        );
    }

    #[test]
    fn test_parse_networks_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
            - https://mainnet.infura.io/v3/1234567890
        chain-id: 1
    testnet:
        rpcs:
            - https://testnet.infura.io
        chain-id: 2
"#;
        let yaml_two = r#"
networks:
    network-one:
        rpcs:
            - https://network-one.infura.io
        chain-id: 3
    network-two:
        rpcs:
            - https://network-two.infura.io
        chain-id: 4
"#;
        let networks = NetworkCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        assert_eq!(networks.len(), 4);
        assert_eq!(
            networks.get("mainnet").unwrap().rpcs,
            vec![
                Url::parse("https://mainnet.infura.io").unwrap(),
                Url::parse("https://mainnet.infura.io/v3/1234567890").unwrap(),
            ]
        );
        assert_eq!(
            networks.get("testnet").unwrap().rpcs,
            vec![Url::parse("https://testnet.infura.io").unwrap()]
        );
        assert_eq!(
            networks.get("network-one").unwrap().rpcs,
            vec![Url::parse("https://network-one.infura.io").unwrap()]
        );
        assert_eq!(
            networks.get("network-two").unwrap().rpcs,
            vec![Url::parse("https://network-two.infura.io").unwrap()]
        );
    }

    #[test]
    fn test_parse_networks_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
    testnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 2
"#;
        let yaml_two = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
"#;
        let error = NetworkCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing("mainnet".to_string(), "networks".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'mainnet' is defined multiple times in your YAML configuration at networks"
        );
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
networks: test
"#;
        let error = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap_err();
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
        assert_eq!(
            error.to_readable_msg(),
            "Field 'networks' in root must be a map"
        );

        let yaml = r#"
networks:
  - test
"#;
        let error = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap_err();
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
        assert_eq!(
            error.to_readable_msg(),
            "Field 'networks' in root must be a map"
        );

        let yaml = r#"
networks:
  - test: test
"#;
        let error = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap_err();
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
        assert_eq!(
            error.to_readable_msg(),
            "Field 'networks' in root must be a map"
        );

        let yaml = r#"
networks:
  mainnet:
    rpcs:
      - https://rpc.com
    chain-id: 1
"#;
        let res = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap();
        assert_eq!(res, vec![Url::parse("https://rpc.com").unwrap()]);
    }

    #[test]
    fn test_to_yaml_hash_serializes_all_fields() {
        let mut networks = HashMap::new();
        networks.insert(
            "sepolia".to_string(),
            NetworkCfg {
                document: Arc::new(RwLock::new(StrictYaml::Hash(Hash::new()))),
                key: "sepolia".to_string(),
                rpcs: vec![Url::parse("https://rpc.sepolia.example").unwrap()],
                chain_id: 11155111,
                label: Some("Ethereum Sepolia".to_string()),
                network_id: Some(11111),
                currency: Some("SEP".to_string()),
            },
        );
        networks.insert(
            "goerli".to_string(),
            NetworkCfg {
                document: Arc::new(RwLock::new(StrictYaml::Hash(Hash::new()))),
                key: "goerli".to_string(),
                rpcs: vec![Url::parse("https://rpc.goerli.example").unwrap()],
                chain_id: 5,
                label: None,
                network_id: None,
                currency: None,
            },
        );

        let yaml = NetworkCfg::to_yaml_hash(&networks).unwrap();

        let StrictYaml::Hash(networks_hash) = yaml else {
            panic!("networks were not serialized to a YAML hash");
        };
        let Some(StrictYaml::Hash(sepolia_hash)) =
            networks_hash.get(&StrictYaml::String("sepolia".to_string()))
        else {
            panic!("sepolia network missing from serialized YAML");
        };
        let Some(StrictYaml::Hash(goerli_hash)) =
            networks_hash.get(&StrictYaml::String("goerli".to_string()))
        else {
            panic!("goerli network missing from serialized YAML");
        };

        assert_eq!(
            sepolia_hash.get(&StrictYaml::String("chain-id".to_string())),
            Some(&StrictYaml::String("11155111".to_string()))
        );
        assert_eq!(
            sepolia_hash.get(&StrictYaml::String("label".to_string())),
            Some(&StrictYaml::String("Ethereum Sepolia".to_string()))
        );
        assert_eq!(
            sepolia_hash.get(&StrictYaml::String("network-id".to_string())),
            Some(&StrictYaml::String("11111".to_string()))
        );
        assert_eq!(
            sepolia_hash.get(&StrictYaml::String("currency".to_string())),
            Some(&StrictYaml::String("SEP".to_string()))
        );
        assert_eq!(
            sepolia_hash.get(&StrictYaml::String("rpcs".to_string())),
            Some(&StrictYaml::Array(vec![StrictYaml::String(
                "https://rpc.sepolia.example/".to_string()
            )]))
        );
        assert_eq!(
            goerli_hash.get(&StrictYaml::String("chain-id".to_string())),
            Some(&StrictYaml::String("5".to_string()))
        );
        assert_eq!(
            goerli_hash.get(&StrictYaml::String("rpcs".to_string())),
            Some(&StrictYaml::Array(vec![StrictYaml::String(
                "https://rpc.goerli.example/".to_string()
            )]))
        );
        assert!(!goerli_hash.contains_key(&StrictYaml::String("label".to_string())));
        assert!(!goerli_hash.contains_key(&StrictYaml::String("network-id".to_string())));
        assert!(!goerli_hash.contains_key(&StrictYaml::String("currency".to_string())));
    }
}
