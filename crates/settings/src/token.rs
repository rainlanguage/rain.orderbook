use crate::yaml::{optional_string, require_hash, require_string, YamlError, YamlParsableHash};
use crate::*;
use alloy::primitives::{hex::FromHexError, Address};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(default)]
pub struct Token {
    #[serde(skip)]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub decimals: Option<u8>,
    pub label: Option<String>,
    pub symbol: Option<String>,
}
impl Token {
    pub fn validate_address(value: &str) -> Result<Address, ParseTokenConfigSourceError> {
        Address::from_str(value).map_err(ParseTokenConfigSourceError::AddressParseError)
    }
    pub fn validate_decimals(value: &str) -> Result<u8, ParseTokenConfigSourceError> {
        value
            .parse::<u8>()
            .map_err(ParseTokenConfigSourceError::DecimalsParseError)
    }

    pub fn update_address(&mut self, address: &str) -> Result<Self, YamlError> {
        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut tokens)) =
                document_hash.get_mut(&StrictYaml::String("tokens".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut token)) =
                    tokens.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    token[&StrictYaml::String("address".to_string())] =
                        StrictYaml::String(address.to_string());
                    self.address = Token::validate_address(address)?;
                } else {
                    return Err(YamlError::ParseError(format!(
                        "missing field: {} in tokens",
                        self.key
                    )));
                }
            } else {
                return Err(YamlError::ParseError("missing field: tokens".to_string()));
            }
        } else {
            return Err(YamlError::ParseError("document parse error".to_string()));
        }

        Ok(self.clone())
    }
}
impl YamlParsableHash for Token {
    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let tokens_hash = require_hash(
            &document_read,
            Some("tokens"),
            Some("missing field: tokens".to_string()),
        )?;

        tokens_hash
            .into_iter()
            .map(|(key_yaml, token_yaml)| {
                let token_key = key_yaml.as_str().unwrap_or_default().to_string();

                let network = Network::parse_from_yaml(
                    document.clone(),
                    require_string(
                        token_yaml,
                        Some("network"),
                        Some(format!("network string missing in token: {token_key}")),
                    )?,
                )
                .map_err(|_| {
                    ParseTokenConfigSourceError::NetworkNotFoundError(token_key.clone())
                })?;

                let address = Token::validate_address(&require_string(
                    token_yaml,
                    Some("address"),
                    Some(format!("address string missing in token: {token_key}")),
                )?)?;

                let decimals = optional_string(token_yaml, "decimals")
                    .map(|d| Token::validate_decimals(&d))
                    .transpose()?;

                let label = optional_string(token_yaml, "label");
                let symbol = optional_string(token_yaml, "symbol");

                let token = Token {
                    document: document.clone(),
                    key: token_key.clone(),
                    network: Arc::new(network),
                    address,
                    decimals,
                    label,
                    symbol,
                };

                Ok((token_key, token))
            })
            .collect()
    }
}

#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Token);

impl Default for Token {
    fn default() -> Self {
        Token {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            network: Arc::new(Network::dummy()),
            address: Address::ZERO,
            decimals: None,
            label: None,
            symbol: None,
        }
    }
}
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.network == other.network
            && self.address == other.address
            && self.decimals == other.decimals
            && self.label == other.label
            && self.symbol == other.symbol
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseTokenConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Failed to parse decimals")]
    DecimalsParseError(std::num::ParseIntError),
    #[error("Network not found for token: {0}")]
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
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: self.network.clone(),
            network: network_ref,
            address: self.address,
            decimals: self.decimals,
            label: self.label,
            symbol: self.symbol,
        })
    }
}

#[cfg(test)]
mod tests {
    use self::test::*;
    use super::*;
    use alloy::primitives::Address;
    use strict_yaml_rust::StrictYamlLoader;

    fn setup_networks() -> HashMap<String, Arc<Network>> {
        let network = mock_network();
        let mut networks = HashMap::new();
        networks.insert("TestNetwork".to_string(), network);
        networks
    }

    fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
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

        assert_eq!(token.key, "TestNetwork");
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

        assert_eq!(token.key, "TestNetwork");
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

    #[test]
    fn test_parse_tokens_errors() {
        let error = Token::parse_all_from_yaml(get_document(
            r#"
test: test
"#,
        ))
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: tokens".to_string())
        );

        let error = Token::parse_all_from_yaml(get_document(
            r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        address: "0x1234567890123456789012345678901234567890"
"#,
        ))
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("network string missing in token: token1".to_string())
        );

        let error = Token::parse_all_from_yaml(get_document(
            r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "nonexistent"
        address: "0x1234567890123456789012345678901234567890"
"#,
        ))
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseTokenConfigSourceError(
                ParseTokenConfigSourceError::NetworkNotFoundError("token1".to_string())
            )
        );

        let error = Token::parse_all_from_yaml(get_document(
            r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "mainnet"
"#,
        ))
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address string missing in token: token1".to_string())
        );

        let error = Token::parse_all_from_yaml(get_document(
            r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "mainnet"
        address: "not_a_valid_address"
"#,
        ));
        assert!(error.is_err());

        let error = Token::parse_all_from_yaml(get_document(
            r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "mainnet"
        address: "0x1234567890123456789012345678901234567890"
        decimals: "not_a_number"
"#,
        ));
        assert!(error.is_err());
    }
}
