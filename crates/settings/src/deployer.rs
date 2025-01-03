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
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let deployers_hash = require_hash(
            &document_read,
            Some("deployers"),
            Some("missing field: deployers".to_string()),
        )?;

        deployers_hash
            .iter()
            .map(|(key_yaml, deployer_yaml)| {
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
                let network = Network::parse_from_yaml(document.clone(), &network_name)?;

                let deployer = Deployer {
                    document: document.clone(),
                    key: deployer_key.clone(),
                    address,
                    network: Arc::new(network),
                };

                Ok((deployer_key, deployer))
            })
            .collect()
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
    fn test_parse_deployers_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Deployer::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: deployers".to_string())
        );

        let yaml = r#"
deployers:
    TestDeployer:
"#;
        let error = Deployer::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address string missing in deployer: TestDeployer".to_string())
        );

        let yaml = r#"
deployers:
    TestDeployer:
        address: not_a_valid_address
"#;
        let error = Deployer::parse_all_from_yaml(get_document(yaml));
        assert!(error.is_err());

        let error = Deployer::parse_all_from_yaml(get_document(
            r#"
networks:
    TestNetwork:
        rpc: https://rpc.com
        chain-id: 1
deployers:
    TestDeployer:
        address: "0x1234567890123456789012345678901234567890"
        network: SomeNetwork
"#,
        ))
        .unwrap_err();
        assert_eq!(error, YamlError::KeyNotFound("SomeNetwork".to_string()));
    }
}
