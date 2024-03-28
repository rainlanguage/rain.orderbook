use crate::config_source::*;
use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainId {
    pub name: String,
    pub chain: String,
    pub icon: Option<String>,
    pub rpc: Vec<String>,
    pub features: Option<Vec<Features>>,
    pub faucets: Option<Vec<String>>,
    pub native_currency: NativeCurrency,
    #[serde(rename = "infoURL")]
    pub info_url: String,
    pub short_name: String,
    pub chain_id: u64,
    pub network_id: u64,
    pub slip44: Option<u64>,
    pub ens: Option<ENS>,
    pub explorers: Option<Vec<Explorer>>,
    pub red_flags: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Features {
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NativeCurrency {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ENS {
    pub registry: Address,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Explorer {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    pub standard: String,
}

#[derive(Error, Debug)]
pub enum ChainIdError {
    #[error("failed to parse provided rpc urls: {:?}", 0)]
    InvalidRpcUrls(Vec<ParseError>),
    #[error("cannot find any rpc urls for this chain")]
    NoRpc,
}

impl TryFrom<ChainId> for NetworkConfigSource {
    type Error = ChainIdError;
    fn try_from(value: ChainId) -> Result<NetworkConfigSource, Self::Error> {
        if value.rpc.is_empty() {
            return Err(ChainIdError::NoRpc);
        }
        let mut errors = vec![];
        for rpc in &value.rpc {
            if !rpc.contains("${") && !rpc.starts_with("ws") {
                match Url::parse(rpc) {
                    Ok(rpc_url) => {
                        return Ok(NetworkConfigSource {
                            chain_id: value.chain_id,
                            rpc: rpc_url,
                            network_id: Some(value.network_id),
                            currency: Some(value.native_currency.symbol),
                            label: Some(value.name),
                        })
                    }
                    Err(e) => errors.push(e),
                }
            }
        }
        Err(ChainIdError::InvalidRpcUrls(errors))
    }
}
