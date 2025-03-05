use crate::yaml::{
    default_document, optional_string, require_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use crate::*;
use alloy::primitives::{hex::FromHexError, Address};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use strict_yaml_rust::strict_yaml::Hash;
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::context::Context;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TokenCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub network: Arc<NetworkCfg>,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub decimals: Option<u8>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub symbol: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TokenCfg);

impl TokenCfg {
    pub fn validate_address(value: &str) -> Result<Address, ParseTokenConfigSourceError> {
        Address::from_str(value).map_err(ParseTokenConfigSourceError::AddressParseError)
    }
    pub fn validate_decimals(value: &str) -> Result<u8, ParseTokenConfigSourceError> {
        value
            .parse::<u8>()
            .map_err(ParseTokenConfigSourceError::DecimalsParseError)
    }

    pub fn update_address(&mut self, address: &str) -> Result<Self, YamlError> {
        let address = TokenCfg::validate_address(address)?;

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
                    self.address = address;
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "tokens".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("tokens".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(self.clone())
    }

    pub async fn add_record_to_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        key: &str,
        network_key: &str,
        address: &str,
        decimals: Option<&str>,
        label: Option<&str>,
        symbol: Option<&str>,
    ) -> Result<(), YamlError> {
        if TokenCfg::parse_from_yaml(documents.clone(), key, None)
            .await
            .is_ok()
        {
            return Err(YamlError::KeyShadowing(key.to_string()));
        }

        let address = TokenCfg::validate_address(address)?;
        let decimals = decimals.map(TokenCfg::validate_decimals).transpose()?;
        NetworkCfg::parse_from_yaml(documents.clone(), network_key, None).await?;

        let mut document = documents[0]
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if !document_hash.contains_key(&StrictYaml::String("tokens".to_string()))
                || document_hash
                    .get_mut(&StrictYaml::String("tokens".to_string()))
                    .is_none()
            {
                document_hash.insert(
                    StrictYaml::String("tokens".to_string()),
                    StrictYaml::Hash(Hash::new()),
                );
            }

            if let Some(StrictYaml::Hash(ref mut tokens)) =
                document_hash.get_mut(&StrictYaml::String("tokens".to_string()))
            {
                if tokens.contains_key(&StrictYaml::String(key.to_string())) {
                    return Err(YamlError::KeyShadowing(key.to_string()));
                }

                let mut token_hash = Hash::new();
                token_hash.insert(
                    StrictYaml::String("network".to_string()),
                    StrictYaml::String(network_key.to_string()),
                );
                token_hash.insert(
                    StrictYaml::String("address".to_string()),
                    StrictYaml::String(address.to_string()),
                );
                if let Some(decimals) = decimals {
                    token_hash.insert(
                        StrictYaml::String("decimals".to_string()),
                        StrictYaml::String(decimals.to_string()),
                    );
                }
                if let Some(label) = label {
                    token_hash.insert(
                        StrictYaml::String("label".to_string()),
                        StrictYaml::String(label.to_string()),
                    );
                }
                if let Some(symbol) = symbol {
                    token_hash.insert(
                        StrictYaml::String("symbol".to_string()),
                        StrictYaml::String(symbol.to_string()),
                    );
                }

                tokens.insert(
                    StrictYaml::String(key.to_string()),
                    StrictYaml::Hash(token_hash),
                );
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("tokens".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(())
    }

    pub fn remove_record_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        key: &str,
    ) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;

            if let StrictYaml::Hash(ref mut document_hash) = *document_write {
                if let Some(StrictYaml::Hash(ref mut tokens)) =
                    document_hash.get_mut(&StrictYaml::String("tokens".to_string()))
                {
                    if tokens.contains_key(&StrictYaml::String(key.to_string())) {
                        tokens.remove(&StrictYaml::String(key.to_string()));
                        return Ok(());
                    }
                }
            }
        }

        Err(YamlError::KeyNotFound(key.to_string()))
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        token_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(tokens_hash) = require_hash(&document_read, Some("tokens"), None) {
                if let Some(token_yaml) =
                    tokens_hash.get(&StrictYaml::String(token_key.to_string()))
                {
                    let location = format!("token '{}'", token_key);
                    return require_string(token_yaml, Some("network"), Some(location));
                }
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("network for token '{}'", token_key)),
            location: "root".to_string(),
        })
    }
}

#[async_trait::async_trait]
impl YamlParsableHash for TokenCfg {
    async fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut tokens = HashMap::new();

        let networks = NetworkCfg::parse_all_from_yaml(documents.clone(), None).await?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(tokens_hash) = require_hash(&document_read, Some("tokens"), None) {
                for (key_yaml, token_yaml) in tokens_hash {
                    let token_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("token '{}'", token_key);

                    let network_key =
                        require_string(token_yaml, Some("network"), Some(location.clone()))?;
                    let network = networks.get(&network_key).ok_or_else(|| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "network".to_string(),
                            reason: format!("Network '{}' not found", network_key),
                        },
                        location: location.clone(),
                    })?;

                    let address_str =
                        require_string(token_yaml, Some("address"), Some(location.clone()))?;
                    let address =
                        TokenCfg::validate_address(&address_str).map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let decimals = optional_string(token_yaml, "decimals")
                        .map(|d| TokenCfg::validate_decimals(&d))
                        .transpose()
                        .map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "decimals".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let label = optional_string(token_yaml, "label");
                    let symbol = optional_string(token_yaml, "symbol");

                    let token = TokenCfg {
                        document: document.clone(),
                        key: token_key.clone(),
                        network: Arc::new(network.clone()),
                        address,
                        decimals,
                        label,
                        symbol,
                    };

                    if tokens.contains_key(&token_key) {
                        return Err(YamlError::KeyShadowing(token_key));
                    }
                    tokens.insert(token_key, token);
                }
            }
        }

        if tokens.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("tokens".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(tokens)
    }
}

impl Default for TokenCfg {
    fn default() -> Self {
        TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            network: Arc::new(NetworkCfg::dummy()),
            address: Address::ZERO,
            decimals: None,
            label: None,
            symbol: None,
        }
    }
}
impl PartialEq for TokenCfg {
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
        name: &str,
        networks: &HashMap<String, Arc<NetworkCfg>>,
    ) -> Result<TokenCfg, ParseTokenConfigSourceError> {
        let network_ref = networks
            .get(&self.network)
            .ok_or(ParseTokenConfigSourceError::NetworkNotFoundError(
                self.network.clone(),
            ))
            .map(Arc::clone)?;

        Ok(TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: name.to_string(),
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
    use yaml::tests::get_document;

    fn setup_networks() -> HashMap<String, Arc<NetworkCfg>> {
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

        let token = token_string.try_into_token("TestNetwork", &networks);

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

        let token = token_string.try_into_token("TestNetwork", &networks);

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

        let token = token_string.try_into_token("TestNetwork", &networks);

        assert!(token.is_err());
        assert_eq!(
            token.unwrap_err(),
            ParseTokenConfigSourceError::NetworkNotFoundError("InvalidNetwork".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_tokens_errors() {
        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
test: test
"#,
            )],
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            }
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
test: test
"#,
            )],
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("tokens".to_string()),
                location: "root".to_string(),
            }
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        address: "0x1234567890123456789012345678901234567890"
"#,
            )],
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("network".to_string()),
                location: "token 'token1'".to_string(),
            }
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
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
            )],
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "network".to_string(),
                    reason: "Network 'nonexistent' not found".to_string(),
                },
                location: "token 'token1'".to_string(),
            }
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "mainnet"
"#,
            )],
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("address".to_string()),
                location: "token 'token1'".to_string(),
            }
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
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
            )],
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { .. },
                location: _
            }
        ));

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
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
            )],
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { .. },
                location: _
            }
        ));
    }

    #[tokio::test]
    async fn test_parse_tokens_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    dai:
        network: mainnet
        address: "0x6b175474e89094c44da98b954eedeac495271d0f"
        decimals: "18"
    usdc:
        network: mainnet
        address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        decimals: "6"
"#;
        let yaml_two = r#"
tokens:
    weth:
        network: mainnet
        address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
        decimals: "18"
    usdt:
        network: mainnet
        address: "0xdac17f958d2ee523a2206206994597c13d831ec7"
        decimals: "6"
"#;
        let tokens = TokenCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .await
        .unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(
            tokens.get("dai").unwrap().address,
            Address::from_str("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap()
        );
        assert_eq!(tokens.get("dai").unwrap().decimals, Some(18));
        assert_eq!(
            tokens.get("usdc").unwrap().address,
            Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap()
        );
        assert_eq!(tokens.get("usdc").unwrap().decimals, Some(6));
        assert_eq!(
            tokens.get("weth").unwrap().address,
            Address::from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()
        );
        assert_eq!(tokens.get("weth").unwrap().decimals, Some(18));
        assert_eq!(
            tokens.get("usdt").unwrap().address,
            Address::from_str("0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap()
        );
        assert_eq!(tokens.get("usdt").unwrap().decimals, Some(6));
    }

    #[tokio::test]
    async fn test_parse_tokens_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    dai:
        network: mainnet
        address: "0x6b175474e89094c44da98b954eedeac495271d0f"
    usdc:
        network: mainnet
        address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
"#;
        let yaml_two = r#"
tokens:
    dai:
        network: mainnet
        address: "0x6b175474e89094c44da98b954eedeac495271d0f"
"#;
        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(error, YamlError::KeyShadowing("dai".to_string()));
    }
}
