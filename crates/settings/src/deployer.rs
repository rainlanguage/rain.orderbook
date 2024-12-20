use crate::*;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct Deployer {
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    pub label: Option<String>,
}
impl Deployer {
    pub fn dummy() -> Self {
        Deployer {
            address: Address::default(),
            network: Arc::new(Network::dummy()),
            label: None,
        }
    }
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Deployer);

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
            address: self.address,
            network: network_ref,
            label: self.label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

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
        assert_eq!(deployer.label, Some("Test Deployer".to_string()));
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
}
