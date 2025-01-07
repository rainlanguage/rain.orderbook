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
use typeshare::typeshare;
use yaml::{
    default_document, optional_string, require_hash, require_string, YamlError, YamlParsableHash,
};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct Deployer {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
}
impl Deployer {
    pub fn dummy() -> Self {
        Deployer {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: Arc::new(Network::dummy()),
        }
    }

    pub fn validate_address(value: &str) -> Result<Address, ParseDeployerConfigSourceError> {
        Address::from_str(value).map_err(ParseDeployerConfigSourceError::AddressParseError)
    }
}

#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Deployer);

impl Default for Deployer {
    fn default() -> Self {
        Deployer::dummy()
    }
}

impl PartialEq for Deployer {
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
        networks: &HashMap<String, Arc<Network>>,
    ) -> Result<Deployer, ParseDeployerConfigSourceError> {
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

        Ok(Deployer {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: name,
            address: self.address,
            network: network_ref,
        })
    }
}

impl YamlParsableHash for Deployer {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut deployers = HashMap::new();

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            let deployers_hash = require_hash(
                &document_read,
                Some("deployers"),
                Some("missing field: deployers".to_string()),
            )?;

            for (key_yaml, deployer_yaml) in deployers_hash {
                let deployer_key = key_yaml.as_str().unwrap_or_default().to_string();

                let address = Deployer::validate_address(&require_string(
                    deployer_yaml,
                    Some("address"),
                    Some(format!(
                        "address string missing in deployer: {deployer_key}"
                    )),
                )?)?;

                let network_name = match optional_string(deployer_yaml, "network") {
                    Some(network_name) => network_name,
                    None => deployer_key.clone(),
                };
                let network = Network::parse_from_yaml(documents.clone(), &network_name)?;

                let deployer = Deployer {
                    document: document.clone(),
                    key: deployer_key.clone(),
                    address,
                    network: Arc::new(network),
                };

                if deployers.contains_key(&deployer_key) {
                    return Err(YamlError::KeyShadowing(deployer_key));
                }
                deployers.insert(deployer_key, deployer);
            }
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
deployers:
    DeployerOne:
        address: 0x1234567890123456789012345678901234567890
        network: NetworkOne
"#;
        let yaml_two = r#"
deployers:
    DeployerTwo:
        address: 0x0987654321098765432109876543210987654321
        network: NetworkTwo
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let deployers = Deployer::parse_all_from_yaml(documents).unwrap();

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
deployers:
    DuplicateDeployer:
        address: 0x1234567890123456789012345678901234567890
        network: NetworkOne
"#;
        let yaml_two = r#"
deployers:
    DuplicateDeployer:
        address: 0x0987654321098765432109876543210987654321
        network: NetworkTwo
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = Deployer::parse_all_from_yaml(documents).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateDeployer".to_string())
        );
    }
}
