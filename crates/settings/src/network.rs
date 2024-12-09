use crate::config_source::*;
use crate::yaml::orderbook::network::NetworkYaml;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
use thiserror::Error;
use typeshare::typeshare;
use url::{ParseError, Url};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct Network {
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
            name: "".to_string(),
            rpc: Url::parse("http://rpc.com").unwrap(),
            chain_id: 1,
            label: None,
            network_id: None,
            currency: None,
        }
    }
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Network);

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
