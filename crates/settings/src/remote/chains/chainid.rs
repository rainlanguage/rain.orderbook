use std::sync::{Arc, RwLock};

use crate::{config_source::*, NetworkCfg};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::Url;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainId {
    pub name: String,
    pub chain: String,
    pub icon: Option<String>,
    pub rpc: Vec<Url>,
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
    pub url: Url,
    pub icon: Option<String>,
    pub standard: String,
}

#[derive(Error, Debug, PartialEq)]
pub enum ChainIdError {
    #[error("provided rpc urls are not supported")]
    UnsupportedRpcUrls,
    #[error("cannot find any rpc urls for this chain")]
    NoRpc,
}

impl TryFrom<ChainId> for NetworkConfigSource {
    type Error = ChainIdError;
    fn try_from(value: ChainId) -> Result<NetworkConfigSource, Self::Error> {
        if value.rpc.is_empty() {
            return Err(ChainIdError::NoRpc);
        }
        for rpc in &value.rpc {
            if !rpc.path().contains("API_KEY") && !rpc.scheme().starts_with("ws") {
                return Ok(NetworkConfigSource {
                    chain_id: value.chain_id,
                    rpc: rpc.clone(),
                    network_id: Some(value.network_id),
                    currency: Some(value.native_currency.symbol),
                    label: Some(value.name),
                });
            }
        }
        Err(ChainIdError::UnsupportedRpcUrls)
    }
}

impl ChainId {
    pub fn try_into_network_cfg(
        self,
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<NetworkCfg, ChainIdError> {
        if self.rpc.is_empty() {
            return Err(ChainIdError::NoRpc);
        }
        for rpc in &self.rpc {
            if !rpc.path().contains("API_KEY") && !rpc.scheme().starts_with("ws") {
                return Ok(NetworkCfg {
                    document: document.clone(),
                    key: self.short_name,
                    rpcs: vec![rpc.clone()],
                    chain_id: self.chain_id,
                    label: Some(self.name),
                    network_id: Some(self.network_id),
                    currency: Some(self.native_currency.symbol),
                });
            }
        }
        Err(ChainIdError::UnsupportedRpcUrls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_err_no_rpc() {
        let chain_id = mk_chain_id_with_rpc(vec![]);

        let network_cfg_source = NetworkConfigSource::try_from(chain_id.clone());
        assert_eq!(network_cfg_source, Err(ChainIdError::NoRpc));

        let strict_yaml = StrictYaml::String("".to_string());
        let strict_yaml_arc = Arc::new(RwLock::new(strict_yaml));

        let network_cfg = chain_id.try_into_network_cfg(strict_yaml_arc);
        assert_eq!(network_cfg, Err(ChainIdError::NoRpc));
    }

    #[test]
    fn test_try_from_err_unsupported_rpc_urls() {
        let rpc = vec![
            Url::parse("https://abcd.com/v3/${API_KEY}").unwrap(),
            Url::parse("wss://example.net/v3/${API_KEY}").unwrap(),
            Url::parse("wss://api.mycryptoapi.com/eth").unwrap(),
        ];

        let chain_id = mk_chain_id_with_rpc(rpc);

        let network_cfg_source = NetworkConfigSource::try_from(chain_id.clone());
        assert_eq!(network_cfg_source, Err(ChainIdError::UnsupportedRpcUrls));

        let strict_yaml = StrictYaml::String("".to_string());
        let strict_yaml_arc = Arc::new(RwLock::new(strict_yaml));

        let network_cfg = chain_id.try_into_network_cfg(strict_yaml_arc);
        assert_eq!(network_cfg, Err(ChainIdError::UnsupportedRpcUrls));
    }

    #[test]
    fn test_try_from_ok() {
        let rpc = vec![
            Url::parse("https://abcd.com/v3/${API_KEY}").unwrap(),
            Url::parse("wss://api.mycryptoapi.com/eth").unwrap(),
            Url::parse("https://cloudflare-eth.com").unwrap(),
        ];

        let chain_id = mk_chain_id_with_rpc(rpc);

        let network_cfg_source = NetworkConfigSource::try_from(chain_id.clone());
        assert_eq!(
            network_cfg_source,
            Ok(NetworkConfigSource {
                chain_id: 1,
                rpc: Url::parse("https://cloudflare-eth.com").unwrap(),
                network_id: Some(1),
                currency: Some("ETH".to_string()),
                label: Some("Ethereum Mainnet".to_string()),
            })
        );

        let strict_yaml = StrictYaml::String("".to_string());
        let strict_yaml_arc = Arc::new(RwLock::new(strict_yaml));

        let network_cfg = chain_id.try_into_network_cfg(strict_yaml_arc.clone());
        assert_eq!(
            network_cfg,
            Ok(NetworkCfg {
                document: strict_yaml_arc,
                key: "short_name".to_string(),
                rpcs: vec![Url::parse("https://cloudflare-eth.com").unwrap()],
                chain_id: 1,
                label: Some("Ethereum Mainnet".to_string()),
                network_id: Some(1),
                currency: Some("ETH".to_string()),
            })
        );
    }

    fn mk_chain_id_with_rpc(rpc: Vec<Url>) -> ChainId {
        ChainId {
            name: "Ethereum Mainnet".to_string(),
            chain: "mainnet".to_string(),
            icon: None,
            rpc,
            features: None,
            faucets: None,
            native_currency: NativeCurrency {
                name: "Ethereum".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            info_url: "info_url".to_string(),
            short_name: "short_name".to_string(),
            chain_id: 1,
            network_id: 1,
            slip44: None,
            ens: None,
            explorers: None,
            red_flags: None,
        }
    }
}
