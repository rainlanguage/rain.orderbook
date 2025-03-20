use crate::{
    yaml::{orderbook::OrderbookYaml, YamlParsable},
    TokenCfg,
};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub chain_id: u64,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub name: String,
    pub timestamp: String,
    pub keywords: Vec<String>,
    pub version: Version,
    pub tokens: Vec<Token>,
    pub logo_uri: String,
}

impl Token {
    pub fn try_into_token_cfg(
        self,
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<TokenCfg, RemoteTokensError> {
        let orderbook_yaml = OrderbookYaml::from_documents(vec![document.clone()]);

        let network = orderbook_yaml
            .get_network_by_chain_id(self.chain_id)
            .map_err(|e| RemoteTokensError::ParseNetworkError(e.to_string()))?;

        let token_cfg = TokenCfg {
            document: document.clone(),
            key: self.name.to_lowercase().replace(" ", "_").clone(),
            network: Arc::new(network),
            address: Address::from_str(&self.address)
                .map_err(|e| RemoteTokensError::ParseTokenAddressError(e.to_string()))?,
            decimals: Some(self.decimals as u8),
            label: Some(self.name.clone()),
            symbol: Some(self.symbol),
        };

        Ok(token_cfg)
    }
}

#[derive(Debug, Error)]
pub enum RemoteTokensError {
    #[error("Failed to parse token address: {0}")]
    ParseTokenAddressError(String),
    #[error("Failed to parse network: {0}")]
    ParseNetworkError(String),
}
