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

impl DeployerConfigSource {
    pub fn try_into_deployer(
        self,
        name: String,
        networks: &HashMap<String, Arc<NetworkCfg>>,
    ) -> Result<DeployerCfg, ParseDeployerConfigSourceError> {
        let network_ref = match self.network {
            Some(network_name) => networks
                .get(&network_name)
                .ok_or(ParseDeployerConfigSourceError::NetworkNotFoundError(
                    network_name.clone(),
                ))
                .map(Arc::clone)?,
            None => networks
                .get(&name)
                .ok_or(ParseDeployerConfigSourceError::NetworkNotFoundError(
                    name.clone(),
                ))
                .map(Arc::clone)?,
        };

        Ok(DeployerCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: name,
            address: self.address,
            network: network_ref,
        })
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
                        return Err(YamlError::KeyShadowing(deployer_key));
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
    use crate::test::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_try_into_deployer_success() {
        let address = Address::repeat_byte(0x01); // Generate a random address for testing
        let network_name = "Local Testnet";
        let networks = HashMap::from([(network_name.to_string(), mock_network())]);
        let deployer_string = DeployerConfigSource {
            address,
            network: Some(network_name.to_string()),
            label: Some("Test Deployer".to_string()),
        };

        let result = deployer_string.try_into_deployer(network_name.to_string(), &networks);
        assert!(result.is_ok());
        let deployer = result.unwrap();
        assert_eq!(deployer.address, address);
        assert_eq!(
            deployer.network.as_ref().label,
            Some(network_name.to_string())
        );
    }

    #[test]
    fn test_try_into_deployer_network_not_found_error() {
        let address = Address::repeat_byte(0x01);
        let invalid_network_name = "unknownnet";
        let networks = HashMap::new(); // Empty networks map
        let deployer_string = DeployerConfigSource {
            address,
            network: Some(invalid_network_name.to_string()),
            label: None,
        };

        let result = deployer_string.try_into_deployer(invalid_network_name.to_string(), &networks);
        assert!(matches!(
            result,
            Err(ParseDeployerConfigSourceError::NetworkNotFoundError(_))
        ));
    }

    #[test]
    fn test_try_into_deployer_no_network_specified() {
        let address = Address::repeat_byte(0x01);
        let network_name = "Local Testnet";
        let networks = HashMap::from([(network_name.to_string(), mock_network())]);
        let deployer_string = DeployerConfigSource {
            address,
            network: None, // No network specified
            label: None,
        };

        // Expecting to use the network name as provided in the name parameter of try_into_deployer
        let result = deployer_string.try_into_deployer(network_name.to_string(), &networks);
        assert!(result.is_ok());
        let deployer = result.unwrap();
        assert_eq!(
            deployer.network.as_ref().label,
            Some(network_name.to_string())
        );
    }

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
            YamlError::KeyShadowing("DuplicateDeployer".to_string())
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
