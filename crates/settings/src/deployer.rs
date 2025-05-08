use crate::*;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::{
    context::Context, default_document, optional_string, require_hash, require_string,
    FieldErrorKind, YamlError, YamlParsableHash,
};

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

    pub fn validate_address(value: &str) -> Result<Address, ParseDeployerCfgError> {
        Address::from_str(value).map_err(ParseDeployerCfgError::AddressParseError)
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
pub enum ParseDeployerCfgError {
    #[error("Failed to parse address")]
    AddressParseError(alloy::primitives::hex::FromHexError),
    #[error("Network not found for Deployer: {0}")]
    NetworkNotFoundError(String),
}

impl ParseDeployerCfgError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseDeployerCfgError::AddressParseError(err) =>
                format!("The deployer address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseDeployerCfgError::NetworkNotFoundError(network) =>
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
        rpc: https://rpc.com
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
        rpc: https://rpc.com
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
        rpc: https://rpc.com
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
        rpc: https://rpc.com
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
        rpc: https://rpc.com
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
        rpc: https://rpc.com
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
}
