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
