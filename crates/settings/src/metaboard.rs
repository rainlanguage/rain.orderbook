use crate::yaml::{
    context::Context, default_document, require_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use alloy::primitives::{hex::FromHexError, Address};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct MetaboardCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub url: Url,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(MetaboardCfg);

impl MetaboardCfg {
    pub fn validate_url(value: &str) -> Result<Url, ParseError> {
        Url::parse(value)
    }

    pub fn validate_address(value: &str) -> Result<Address, FromHexError> {
        Address::from_str(value)
    }

    fn metaboard_yaml_value(url: Url, address: Address) -> StrictYaml {
        let mut metaboard_hash = Hash::new();
        metaboard_hash.insert(
            StrictYaml::String("url".to_string()),
            StrictYaml::String(url.to_string()),
        );
        metaboard_hash.insert(
            StrictYaml::String("address".to_string()),
            StrictYaml::String(address.to_string()),
        );
        StrictYaml::Hash(metaboard_hash)
    }

    pub fn add_record_to_yaml(
        document: Arc<RwLock<StrictYaml>>,
        key: &str,
        url: &str,
        address: &str,
    ) -> Result<(), YamlError> {
        let url = MetaboardCfg::validate_url(url).map_err(|e| YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: "url".to_string(),
                reason: e.to_string(),
            },
            location: format!("metaboard '{}'", key),
        })?;
        let address = MetaboardCfg::validate_address(address).map_err(|e| YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: "address".to_string(),
                reason: e.to_string(),
            },
            location: format!("metaboard '{}'", key),
        })?;

        let mut document = document.write().map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if !document_hash.contains_key(&StrictYaml::String("metaboards".to_string())) {
                document_hash.insert(
                    StrictYaml::String("metaboards".to_string()),
                    StrictYaml::Hash(Hash::new()),
                );
            }

            if let Some(StrictYaml::Hash(ref mut metaboards)) =
                document_hash.get_mut(&StrictYaml::String("metaboards".to_string()))
            {
                if metaboards.contains_key(&StrictYaml::String(key.to_string())) {
                    return Err(YamlError::KeyShadowing(
                        key.to_string(),
                        "metaboards".to_string(),
                    ));
                }

                metaboards.insert(
                    StrictYaml::String(key.to_string()),
                    MetaboardCfg::metaboard_yaml_value(url, address),
                );
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("metaboards".to_string()),
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

    pub fn set_record_to_yaml(
        document: Arc<RwLock<StrictYaml>>,
        key: &str,
        url: &str,
        address: &str,
    ) -> Result<(), YamlError> {
        let url = MetaboardCfg::validate_url(url).map_err(|e| YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: "url".to_string(),
                reason: e.to_string(),
            },
            location: format!("metaboard '{}'", key),
        })?;
        let address = MetaboardCfg::validate_address(address).map_err(|e| YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: "address".to_string(),
                reason: e.to_string(),
            },
            location: format!("metaboard '{}'", key),
        })?;

        let mut document = document.write().map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if !document_hash.contains_key(&StrictYaml::String("metaboards".to_string())) {
                document_hash.insert(
                    StrictYaml::String("metaboards".to_string()),
                    StrictYaml::Hash(Hash::new()),
                );
            }

            if let Some(StrictYaml::Hash(ref mut metaboards)) =
                document_hash.get_mut(&StrictYaml::String("metaboards".to_string()))
            {
                metaboards.insert(
                    StrictYaml::String(key.to_string()),
                    MetaboardCfg::metaboard_yaml_value(url, address),
                );
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("metaboards".to_string()),
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
}

impl YamlParsableHash for MetaboardCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, MetaboardCfg>, YamlError> {
        let mut metaboards = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(metaboards_hash) = require_hash(&document_read, Some("metaboards"), None) {
                for (key_yaml, metaboard_yaml) in metaboards_hash {
                    let metaboard_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("metaboard '{}'", metaboard_key);
                    let metaboard_map = require_hash(metaboard_yaml, None, Some(location.clone()))?;

                    let url_str = require_string(
                        metaboard_map
                            .get(&StrictYaml::String("url".to_string()))
                            .ok_or_else(|| YamlError::Field {
                                kind: FieldErrorKind::Missing("url".to_string()),
                                location: location.clone(),
                            })?,
                        None,
                        Some(location.clone()),
                    )?;
                    let url =
                        MetaboardCfg::validate_url(&url_str).map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "url".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;
                    let address_str = require_string(
                        metaboard_map
                            .get(&StrictYaml::String("address".to_string()))
                            .ok_or_else(|| YamlError::Field {
                                kind: FieldErrorKind::Missing("address".to_string()),
                                location: location.clone(),
                            })?,
                        None,
                        Some(location.clone()),
                    )?;
                    let address = MetaboardCfg::validate_address(&address_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let metaboard = MetaboardCfg {
                        document: document.clone(),
                        key: metaboard_key.clone(),
                        url,
                        address,
                    };

                    if metaboards.contains_key(&metaboard_key) {
                        return Err(YamlError::KeyShadowing(
                            metaboard_key,
                            "metaboards".to_string(),
                        ));
                    }
                    metaboards.insert(metaboard_key, metaboard);
                }
            }
        }

        if metaboards.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("metaboards".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(metaboards)
    }
}

impl Default for MetaboardCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            url: Url::parse("https://metaboard.com").unwrap(),
            address: Address::default(),
        }
    }
}

impl PartialEq for MetaboardCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.url == other.url && self.address == other.address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_metaboards_from_yaml_multiple_files() {
        let yaml_one = r#"
metaboards:
    MetaboardOne:
        url: https://metaboard-one.com
        address: 0x1111111111111111111111111111111111111111
"#;
        let yaml_two = r#"
metaboards:
    MetaboardTwo:
        url: https://metaboard-two.com
        address: 0x2222222222222222222222222222222222222222
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let metaboards = MetaboardCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(metaboards.len(), 2);
        assert!(metaboards.contains_key("MetaboardOne"));
        assert!(metaboards.contains_key("MetaboardTwo"));

        assert_eq!(
            metaboards.get("MetaboardOne").unwrap().url,
            Url::parse("https://metaboard-one.com").unwrap()
        );
        assert_eq!(
            metaboards.get("MetaboardTwo").unwrap().url,
            Url::parse("https://metaboard-two.com").unwrap()
        );
        assert_eq!(
            metaboards.get("MetaboardOne").unwrap().address,
            Address::from_str("0x1111111111111111111111111111111111111111").unwrap()
        );
        assert_eq!(
            metaboards.get("MetaboardTwo").unwrap().address,
            Address::from_str("0x2222222222222222222222222222222222222222").unwrap()
        );
    }

    #[test]
    fn test_parse_metaboards_from_yaml_duplicate_key() {
        let yaml_one = r#"
metaboards:
    DuplicateMetaboard:
        url: https://metaboard-one.com
        address: 0x1111111111111111111111111111111111111111
"#;
        let yaml_two = r#"
metaboards:
    DuplicateMetaboard:
        url: https://metaboard-two.com
        address: 0x2222222222222222222222222222222222222222
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = MetaboardCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateMetaboard".to_string(), "metaboards".to_string())
        );
    }

    #[test]
    fn test_parse_metaboards_missing_address_fails() {
        let yaml = r#"
metaboards:
    MissingAddress:
        url: https://metaboard-one.com
"#;

        let error = MetaboardCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("address".to_string()),
                location: "metaboard 'MissingAddress'".to_string(),
            }
        );
    }
}
