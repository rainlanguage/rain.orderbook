use crate::config_source::*;
use serde::{Deserialize, Serialize};
use std::num::ParseIntError;
use thiserror::Error;
use typeshare::typeshare;
use url::{ParseError, Url};

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    #[typeshare(typescript(type = "string"))]
    pub rpc: Url,
    #[typeshare(typescript(type = "number"))]
    pub chain_id: u64,
    pub label: Option<String>,
    #[typeshare(typescript(type = "number"))]
    pub network_id: Option<u64>,
    pub currency: Option<String>,
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

impl TryFrom<NetworkConfigSource> for Network {
    type Error = ParseNetworkConfigSourceError;

    fn try_from(item: NetworkConfigSource) -> Result<Self, Self::Error> {
        Ok(Network {
            rpc: item.rpc,
            chain_id: item.chain_id,
            label: item.label,
            network_id: item.network_id,
            currency: item.currency,
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

        let result = Network::try_from(network_string);
        assert!(result.is_ok());
        let network = result.unwrap();

        assert_eq!(network.rpc, Url::parse("http://127.0.0.1:8545").unwrap());
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.label, Some("Local Testnet".into()));
        assert_eq!(network.currency, Some("ETH".into()));
    }
}
