use crate::yaml::orderbook::network::NetworkYaml;
use crate::{config_source::*, yaml::YamlError};
use serde::{Deserialize, Serialize};
use std::{
    num::ParseIntError,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use typeshare::typeshare;
use url::{ParseError, Url};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Network {
    #[serde(skip)]
    pub document: Arc<RwLock<StrictYaml>>,
    pub name: String,
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub rpc: Url,
    #[typeshare(typescript(type = "number"))]
    pub chain_id: u64,
    pub label: Option<String>,
    #[typeshare(typescript(type = "number"))]
    pub network_id: Option<u64>,
    pub currency: Option<String>,
}
impl Network {
    pub fn dummy() -> Self {
        Network {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            name: "".to_string(),
            rpc: Url::parse("http://rpc.com").unwrap(),
            chain_id: 1,
            label: None,
            network_id: None,
            currency: None,
        }
    }

    pub fn update_rpc(&mut self, rpc: &str) -> Result<(), YamlError> {
        self.rpc = Url::parse(rpc).map_err(ParseNetworkYamlError::RpcParseError)?;
        NetworkYaml::update_rpc(&self.document, &self.name, rpc)?;
        Ok(())
    }
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Network);

impl Default for Network {
    fn default() -> Self {
        Network::dummy()
    }
}
impl PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.rpc == other.rpc
            && self.chain_id == other.chain_id
            && self.label == other.label
            && self.network_id == other.network_id
            && self.currency == other.currency
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseNetworkConfigSourceError {
    #[error("Failed to parse rpc: {}", 0)]
    RpcParseError(ParseError),
    #[error("Failed to parse chain_id: {}", 0)]
    ChainIdParseError(ParseIntError),
    #[error("Failed to parse network_id: {}", 0)]
    NetworkIdParseError(ParseIntError),
}

impl NetworkConfigSource {
    pub fn try_into_network(self, name: String) -> Result<Network, ParseNetworkConfigSourceError> {
        Ok(Network {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            name,
            rpc: self.rpc,
            chain_id: self.chain_id,
            label: self.label,
            network_id: self.network_id,
            currency: self.currency,
        })
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseNetworkYamlError {
    #[error("Failed to parse rpc: {}", 0)]
    RpcParseError(ParseError),
    #[error("Failed to parse chain_id: {}", 0)]
    ChainIdParseError(ParseIntError),
    #[error("Failed to parse network_id: {}", 0)]
    NetworkIdParseError(ParseIntError),
}
impl NetworkYaml {
    pub fn try_into_network(self, name: &str) -> Result<Network, ParseNetworkYamlError> {
        Ok(Network {
            document: self.document,
            name: name.to_string(),
            rpc: Url::parse(&self.rpc).map_err(ParseNetworkYamlError::RpcParseError)?,
            chain_id: self
                .chain_id
                .parse::<u64>()
                .map_err(ParseNetworkYamlError::ChainIdParseError)?,
            label: self.label,
            network_id: self
                .network_id
                .map(|id| id.parse::<u64>())
                .transpose()
                .map_err(ParseNetworkYamlError::NetworkIdParseError)?,
            currency: self.currency,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_try_from_network_string_success() {
        let network_string = NetworkConfigSource {
            rpc: Url::parse("http://127.0.0.1:8545").unwrap(),
            chain_id: 1,
            network_id: Some(1),
            label: Some("Local Testnet".into()),
            currency: Some("ETH".into()),
        };

        let result = network_string.try_into_network("local".into());
        assert!(result.is_ok());
        let network = result.unwrap();

        assert_eq!(network.rpc, Url::parse("http://127.0.0.1:8545").unwrap());
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.label, Some("Local Testnet".into()));
        assert_eq!(network.currency, Some("ETH".into()));
        assert_eq!(network.name, "local");
    }

    #[test]
    fn test_try_from_network_yaml_success() {
        let network_yaml = NetworkYaml {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            rpc: "http://127.0.0.1:8545".to_string(),
            chain_id: "1".to_string(),
            label: Some("Local Testnet".into()),
            network_id: Some("1".to_string()),
            currency: Some("ETH".into()),
        };

        let result = network_yaml.try_into_network("local");
        assert!(result.is_ok());
        let network = result.unwrap();

        assert_eq!(network.rpc, Url::parse("http://127.0.0.1:8545").unwrap());
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.label, Some("Local Testnet".into()));
        assert_eq!(network.currency, Some("ETH".into()));
        assert_eq!(network.name, "local");
    }
}
