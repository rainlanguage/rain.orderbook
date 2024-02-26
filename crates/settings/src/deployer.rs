use crate::*;
use alloy_primitives::Address;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Deployer {
    pub address: Address,
    pub network: Option<Arc<Network>>,
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
            network: Some(network_ref),
            label: self.label,
        })
    }
}
