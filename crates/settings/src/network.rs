use std::num::ParseIntError;

use crate::string_structs::*;
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug, PartialEq)]
pub struct Network {
    pub rpc: Url,
    pub chain_id: u64,
    pub label: Option<String>,
    pub network_id: Option<u64>,
    pub currency: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseNetworkStringError {
    #[error("Failed to parse rpc: {}", 0)]
    RpcParseError(ParseError),
    #[error("Failed to parse chain_id: {}", 0)]
    ChainIdParseError(ParseIntError),
    #[error("Failed to parse network_id: {}", 0)]
    NetworkIdParseError(ParseIntError),
}

impl TryFrom<NetworkString> for Network {
    type Error = ParseNetworkStringError;

    fn try_from(item: NetworkString) -> Result<Self, Self::Error> {
        Ok(Network {
            rpc: item
                .rpc
                .parse()
                .map_err(ParseNetworkStringError::RpcParseError)?,
            chain_id: item
                .chain_id
                .parse()
                .map_err(ParseNetworkStringError::ChainIdParseError)?,
            label: item.label,
            network_id: item
                .network_id
                .map(|id| {
                    id.parse()
                        .map_err(ParseNetworkStringError::NetworkIdParseError)
                })
                .transpose()?,
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
        let network_string = NetworkString {
            rpc: "http://127.0.0.1:8545".into(),
            chain_id: "1".into(),
            network_id: Some("1".into()),
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

    #[test]
    fn test_try_from_network_string_rpc_parse_error() {
        let invalid_rpc = "invalid_url"; // Intentionally invalid URL
        let network_string = NetworkString {
            rpc: invalid_rpc.into(),
            chain_id: "1".into(),
            network_id: Some("1".into()),
            label: None,
            currency: None,
        };

        let result = Network::try_from(network_string);
        assert!(matches!(
            result,
            Err(ParseNetworkStringError::RpcParseError(_))
        ));
    }

    #[test]
    fn test_try_from_network_string_chain_id_parse_error() {
        let invalid_chain_id = "abc"; // Intentionally invalid number format
        let network_string = NetworkString {
            rpc: "http://127.0.0.1:8545".into(),
            chain_id: invalid_chain_id.into(),
            network_id: Some("1".into()),
            label: None,
            currency: None,
        };

        let result = Network::try_from(network_string);
        assert!(matches!(
            result,
            Err(ParseNetworkStringError::ChainIdParseError(_))
        ));
    }

    #[test]
    fn test_try_from_network_string_network_id_parse_error() {
        let invalid_network_id = "abc"; // Intentionally invalid number format
        let network_string = NetworkString {
            rpc: "http://127.0.0.1:8545".into(),
            chain_id: "1".into(),
            network_id: Some(invalid_network_id.into()),
            label: None,
            currency: None,
        };

        let result = Network::try_from(network_string);
        assert!(matches!(
            result,
            Err(ParseNetworkStringError::NetworkIdParseError(_))
        ));
    }
}
