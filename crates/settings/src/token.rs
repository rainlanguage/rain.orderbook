use crate::*;
use alloy::primitives::{hex::FromHexError, Address};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct Token {
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub decimals: Option<u8>,
    pub label: Option<String>,
    pub symbol: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Token);

#[derive(Error, Debug, PartialEq)]
pub enum ParseTokenConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Failed to parse decimals")]
    DecimalsParseError(std::num::ParseIntError),
    #[error("Network not found for Token: {0}")]
    NetworkNotFoundError(String),
}

impl TokenConfigSource {
    pub fn try_into_token(
        self,
        networks: &HashMap<String, Arc<Network>>,
    ) -> Result<Token, ParseTokenConfigSourceError> {
        let network_ref = networks
            .get(&self.network)
            .ok_or(ParseTokenConfigSourceError::NetworkNotFoundError(
                self.network.clone(),
            ))
            .map(Arc::clone)?;

        Ok(Token {
            network: network_ref,
            address: self.address,
            decimals: self.decimals,
            label: self.label,
            symbol: self.symbol,
        })
    }
}

#[cfg(test)]
mod token_tests {
    use self::test::*;
    use super::*;
    use alloy::primitives::Address;

    fn setup_networks() -> HashMap<String, Arc<Network>> {
        let network = mock_network();
        let mut networks = HashMap::new();
        networks.insert("TestNetwork".to_string(), network);
        networks
    }

    #[test]
    fn test_token_creation_success_with_all_fields() {
        let networks = setup_networks();
        let token_string = TokenConfigSource {
            network: "TestNetwork".to_string(),
            address: Address::repeat_byte(0x01),
            decimals: Some(18),
            label: Some("TestToken".to_string()),
            symbol: Some("TTK".to_string()),
        };

        let token = token_string.try_into_token(&networks);

        assert!(token.is_ok());
        let token = token.unwrap();

        assert_eq!(
            Arc::as_ptr(&token.network),
            Arc::as_ptr(networks.get("TestNetwork").unwrap())
        );
        assert_eq!(token.address, Address::repeat_byte(0x01));
        assert_eq!(token.decimals, Some(18));
        assert_eq!(token.label, Some("TestToken".to_string()));
        assert_eq!(token.symbol, Some("TTK".to_string()));
    }

    #[test]
    fn test_token_creation_success_with_minimal_fields() {
        let networks = setup_networks();
        let token_string = TokenConfigSource {
            network: "TestNetwork".to_string(),
            address: Address::repeat_byte(0x01),
            decimals: None,
            label: None,
            symbol: None,
        };

        let token = token_string.try_into_token(&networks);

        assert!(token.is_ok());
        let token = token.unwrap();

        assert_eq!(
            Arc::as_ptr(&token.network),
            Arc::as_ptr(networks.get("TestNetwork").unwrap())
        );
        assert_eq!(token.address, Address::repeat_byte(0x01));
        assert_eq!(token.decimals, None);
        assert_eq!(token.label, None);
        assert_eq!(token.symbol, None);
    }

    #[test]
    fn test_token_creation_failure_due_to_invalid_network() {
        let networks = setup_networks();
        let token_string = TokenConfigSource {
            network: "InvalidNetwork".to_string(),
            address: Address::repeat_byte(0x01),
            decimals: None,
            label: None,
            symbol: None,
        };

        let token = token_string.try_into_token(&networks);

        assert!(token.is_err());
        assert_eq!(
            token.unwrap_err(),
            ParseTokenConfigSourceError::NetworkNotFoundError("InvalidNetwork".to_string())
        );
    }
}
