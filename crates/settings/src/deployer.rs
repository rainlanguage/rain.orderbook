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
    default_document, optional_string, require_hash, require_string, YamlError,
    YamlParsableMergableHash,
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

impl YamlParsableMergableHash for Deployer {
    fn parse_and_merge_all_from_yamls(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut all_deployers = HashMap::new();

        // First get all networks from all documents
        let all_networks = Network::parse_and_merge_all_from_yamls(documents.clone())?;

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
            if let Ok(deployers_hash) = require_hash(
                &document_read,
                Some("deployers"),
                None, // Don't error if not found
            ) {
                for (key_yaml, deployer_yaml) in deployers_hash {
                    let deployer_key = key_yaml.as_str().unwrap_or_default().to_string();

                    // Error on duplicates
                    if all_deployers.contains_key(&deployer_key) {
                        return Err(YamlError::DuplicateKey(deployer_key));
                    }

                    let address = Deployer::validate_address(&require_string(
                        deployer_yaml,
                        Some("address"),
                        Some(format!(
                            "address string missing in deployer: {deployer_key}"
                        )),
                    )?)?;

                    // Get network name from field or use deployer key as fallback
                    let network_name = optional_string(deployer_yaml, "network")
                        .unwrap_or_else(|| deployer_key.clone());

                    let network = all_networks
                        .get(&network_name)
                        .ok_or_else(|| {
                            YamlError::ParseError(format!(
                                "network not found for deployer: {deployer_key}"
                            ))
                        })?
                        .clone();

                    let deployer = Deployer {
                        document: document.clone(),
                        key: deployer_key.clone(),
                        address,
                        network: Arc::new(network),
                    };

                    all_deployers.insert(deployer_key, deployer);
                }
            }
        }

        if all_deployers.is_empty() {
            return Err(YamlError::ParseError(
                "missing field: deployers".to_string(),
            ));
        }

        Ok(all_deployers)
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
        let error = Deployer::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: deployers".to_string())
        );

        let yaml = r#"
deployers:
    TestDeployer:
"#;
        let error = Deployer::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address string missing in deployer: TestDeployer".to_string())
        );

        let yaml = r#"
deployers:
    TestDeployer:
        address: not_a_valid_address
"#;
        let error = Deployer::parse_and_merge_all_from_yamls(vec![get_document(yaml)]);
        assert!(error.is_err());

        let error = Deployer::parse_and_merge_all_from_yamls(vec![get_document(
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
        )])
        .unwrap_err();
        assert_eq!(error, YamlError::KeyNotFound("SomeNetwork".to_string()));
    }

    #[test]
    fn test_deployer_document_preservation() {
        // Main document with one deployer
        let main_yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
deployers:
    deployer1:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
"#;
        let main_doc = get_document(main_yaml);

        // Orderbook yaml with another deployer
        let orderbook_yaml = r#"
networks:
    testnet:
        rpc: https://testnet.infura.io
        chain-id: 5
deployers:
    deployer2:
        address: 0x0987654321098765432109876543210987654321
        network: testnet
"#;
        let orderbook_doc = get_document(orderbook_yaml);

        // Parse both documents
        let deployers =
            Deployer::parse_and_merge_all_from_yamls(vec![main_doc.clone(), orderbook_doc.clone()])
                .unwrap();

        // Verify deployers came from correct documents
        let deployer1 = deployers.get("deployer1").unwrap();
        let deployer2 = deployers.get("deployer2").unwrap();

        // Check document preservation by comparing Arc pointers
        assert!(Arc::ptr_eq(&deployer1.document, &main_doc));
        assert!(Arc::ptr_eq(&deployer2.document, &orderbook_doc));

        // Verify networks were correctly merged and assigned
        assert_eq!(deployer1.network.chain_id, 1);
        assert_eq!(deployer2.network.chain_id, 5);
    }

    #[test]
    fn test_deployer_duplicate_error() {
        let yaml1 = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
deployers:
    deployer1:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
"#;
        let yaml2 = r#"
networks:
    testnet:
        rpc: https://testnet.infura.io
        chain-id: 5
deployers:
    deployer1:
        address: 0x0987654321098765432109876543210987654321
        network: testnet
"#;

        let error = Deployer::parse_and_merge_all_from_yamls(vec![
            get_document(yaml1),
            get_document(yaml2),
        ])
        .unwrap_err();

        assert_eq!(error, YamlError::DuplicateKey("deployer1".to_string()));
    }

    #[test]
    fn test_deployer_network_fallback() {
        let yaml = r#"
networks:
    deployer1:
        rpc: https://mainnet.infura.io
        chain-id: 1
deployers:
    deployer1:
        address: 0x1234567890123456789012345678901234567890
"#;
        let deployers = Deployer::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap();

        let deployer = deployers.get("deployer1").unwrap();
        assert_eq!(deployer.network.chain_id, 1);
        assert_eq!(deployer.network.key, "deployer1");
    }
}
