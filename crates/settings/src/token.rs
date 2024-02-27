use crate::*;
use alloy_primitives::{hex::FromHexError, Address};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod token_tests {
    use self::test::*;
    use super::*;
    use alloy_primitives::Address;

    fn setup_networks() -> HashMap<String, Arc<Network>> {
        let network = mock_network();
        let mut networks = HashMap::new();
        networks.insert("TestNetwork".to_string(), network);
        networks
    }

    #[test]
    fn test_token_creation_success_with_all_fields() {
        let networks = setup_networks();
        let token_string = TokenString {
            network: "TestNetwork".to_string(),
            address: Address::repeat_byte(0x01).to_string(),
            decimals: Some("18".to_string()),
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
        let token_string = TokenString {
            network: "TestNetwork".to_string(),
            address: Address::repeat_byte(0x01).to_string(),
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
        let token_string = TokenString {
            network: "InvalidNetwork".to_string(),
            address: "0x1234".to_string(),
            decimals: None,
            label: None,
            symbol: None,
        };

        let token = token_string.try_into_token(&networks);

        assert!(token.is_err());
        assert_eq!(
            token.unwrap_err(),
            ParseTokenStringError::NetworkNotFoundError("InvalidNetwork".to_string())
        );
    }

    #[test]
    fn test_token_creation_failure_due_to_invalid_address() {
        let networks = setup_networks();
        let token_string = TokenString {
            network: "TestNetwork".to_string(),
            address: "invalid".to_string(),
            decimals: None,
            label: None,
            symbol: None,
        };

        let token = token_string.try_into_token(&networks);

        assert!(token.is_err());
        match token.unwrap_err() {
            ParseTokenStringError::AddressParseError(_) => (),
            _ => panic!("Expected AddressParseError"),
        }
    }

    #[test]
    fn test_token_creation_failure_due_to_invalid_decimals() {
        let networks = setup_networks();
        let token_string = TokenString {
            network: "TestNetwork".to_string(),
            address: Address::repeat_byte(0x03).to_string(),
            decimals: Some("invalid".to_string()),
            label: None,
            symbol: None,
        };

        let token = token_string.try_into_token(&networks);

        assert!(token.is_err());
        match token.unwrap_err() {
            ParseTokenStringError::DecimalsParseError(_) => (),
            _ => panic!("Expected DecimalsParseError"),
        }
    }
}
