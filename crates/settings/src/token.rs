use crate::*;
use alloy_primitives::{hex::FromHexError, Address};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Debug)]
pub struct Token {
    pub network: Arc<Network>,
    pub address: Address,
    pub decimals: Option<u8>,
    pub label: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseTokenStringError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Failed to parse decimals")]
    DecimalsParseError(std::num::ParseIntError),
    #[error("Network not found: {0}")]
    NetworkNotFoundError(String),
}

impl TokenString {
    pub fn try_into_token(
        self,
        networks: &HashMap<String, Arc<Network>>,
    ) -> Result<Token, ParseTokenStringError> {
        let network_ref = networks
            .get(&self.network)
            .ok_or(ParseTokenStringError::NetworkNotFoundError(
                self.network.clone(),
            ))
            .map(Arc::clone)?;

        Ok(Token {
            network: network_ref,
            address: self
                .address
                .parse()
                .map_err(ParseTokenStringError::AddressParseError)?,
            decimals: self
                .decimals
                .map(|decimals| {
                    decimals
                        .parse()
                        .map_err(ParseTokenStringError::DecimalsParseError)
                })
                .transpose()?,
            label: self.label,
            symbol: self.symbol,
        })
    }
}
