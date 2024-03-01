use crate::*;
use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Deployer {
    #[typeshare(typescript(type = "string"))]
    pub address: Address,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    pub label: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseDeployerStringError {
    #[error("Failed to parse address")]
    AddressParseError(alloy_primitives::hex::FromHexError),
    #[error("Network not found: {0}")]
    NetworkNotFoundError(String),
}

impl DeployerString {
    pub fn try_into_deployer(
        self,
        name: String,
        networks: &HashMap<String, Arc<Network>>,
    ) -> Result<Deployer, ParseDeployerStringError> {
        let network_ref = match self.network {
            Some(network_name) => networks
                .get(&network_name)
                .ok_or(ParseDeployerStringError::NetworkNotFoundError(
                    network_name.clone(),
                ))
                .map(Arc::clone)?,
            None => networks
                .get(&name)
                .ok_or(ParseDeployerStringError::NetworkNotFoundError(name.clone()))
                .map(Arc::clone)?,
        };

        Ok(Deployer {
            address: self
                .address
                .parse()
                .map_err(ParseDeployerStringError::AddressParseError)?,
            network: network_ref,
            label: self.label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    // Mock a simple Network struct for testing purposes
    #[derive(Debug, Clone)]
    struct MockNetwork {
        name: String,
    }

    impl PartialEq for MockNetwork {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }

    #[test]
    fn test_try_into_deployer_success() {
        let address = Address::repeat_byte(0x01); // Generate a random address for testing
        let network_name = "Local Testnet";
        let networks = HashMap::from([(network_name.to_string(), mock_network())]);
        let deployer_string = DeployerString {
            address: address.to_string(),
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
        assert_eq!(deployer.label, Some("Test Deployer".to_string()));
    }

    #[test]
    fn test_try_into_deployer_address_parse_error() {
        let invalid_address = "zzz"; // Intentionally invalid address format
        let network_name = "testnet";
        let networks = HashMap::from([(network_name.to_string(), mock_network())]);
        let deployer_string = DeployerString {
            address: invalid_address.into(),
            network: Some(network_name.to_string()),
            label: Some("Invalid Deployer".to_string()),
        };

        let result = deployer_string.try_into_deployer(network_name.to_string(), &networks);
        assert!(matches!(
            result,
            Err(ParseDeployerStringError::AddressParseError(_))
        ));
    }

    #[test]
    fn test_try_into_deployer_network_not_found_error() {
        let address = Address::repeat_byte(0x01);
        let invalid_network_name = "unknownnet";
        let networks = HashMap::new(); // Empty networks map
        let deployer_string = DeployerString {
            address: address.to_string(),
            network: Some(invalid_network_name.to_string()),
            label: None,
        };

        let result = deployer_string.try_into_deployer(invalid_network_name.to_string(), &networks);
        assert!(matches!(
            result,
            Err(ParseDeployerStringError::NetworkNotFoundError(_))
        ));
    }

    #[test]
    fn test_try_into_deployer_no_network_specified() {
        let address = Address::repeat_byte(0x01);
        let network_name = "Local Testnet";
        let networks = HashMap::from([(network_name.to_string(), mock_network())]);
        let deployer_string = DeployerString {
            address: address.to_string(),
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
}
