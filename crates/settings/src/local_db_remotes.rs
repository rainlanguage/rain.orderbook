use crate::yaml::{
    context::Context, default_document, optional_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct LocalDbRemoteCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub url: Url,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(LocalDbRemoteCfg);

impl LocalDbRemoteCfg {
    pub fn validate_url(value: &str) -> Result<Url, ParseError> {
        Url::parse(value)
    }
}

impl YamlParsableHash for LocalDbRemoteCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut remotes = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(remotes_hash) = optional_hash(&document_read, "local-db-remotes") {
                for (key_yaml, remote_yaml) in remotes_hash {
                    let remote_key = key_yaml
                        .as_str()
                        .ok_or(YamlError::Field {
                            kind: FieldErrorKind::InvalidType {
                                field: "key".to_string(),
                                expected: "a string".to_string(),
                            },
                            location: "local-db-remotes".to_string(),
                        })?
                        .to_string();
                    let location = format!("local-db-remotes[{}]", remote_key);

                    let url_str = require_string(remote_yaml, None, Some(location.clone()))?;
                    let url =
                        LocalDbRemoteCfg::validate_url(&url_str).map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "url".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let remote = LocalDbRemoteCfg {
                        document: document.clone(),
                        key: remote_key.clone(),
                        url,
                    };

                    if remotes.contains_key(&remote_key) {
                        return Err(YamlError::KeyShadowing(
                            remote_key,
                            "local-db-remotes".to_string(),
                        ));
                    }
                    remotes.insert(remote.key.clone(), remote);
                }
            }
        }

        Ok(remotes)
    }

    fn to_yaml_value(&self) -> Result<StrictYaml, YamlError> {
        Ok(StrictYaml::String(self.url.to_string()))
    }
}

impl Default for LocalDbRemoteCfg {
    fn default() -> Self {
        Self {
            document: default_document(),
            key: "".to_string(),
            url: Url::parse("https://example.com/localdb").unwrap(),
        }
    }
}

impl PartialEq for LocalDbRemoteCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.url == other.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;
    use std::collections::HashMap;

    #[test]
    fn test_parse_local_db_remotes_from_yaml_multiple_files() {
        let yaml_one = r#"
local-db-remotes:
    mainnet: https://example.com/localdb/mainnet
"#;
        let yaml_two = r#"
local-db-remotes:
    polygon: https://example.com/localdb/polygon
"#;

        let remotes = LocalDbRemoteCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        assert_eq!(remotes.len(), 2);
        assert_eq!(
            remotes.get("mainnet").unwrap().url,
            Url::parse("https://example.com/localdb/mainnet").unwrap()
        );
        assert_eq!(
            remotes.get("polygon").unwrap().url,
            Url::parse("https://example.com/localdb/polygon").unwrap()
        );
    }

    #[test]
    fn test_parse_local_db_remotes_from_yaml_duplicate_key() {
        let yaml_one = r#"
local-db-remotes:
    mainnet: https://example.com/localdb/mainnet
"#;
        let yaml_two = r#"
local-db-remotes:
    mainnet: https://example.com/localdb/another
"#;
        let err = LocalDbRemoteCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            YamlError::KeyShadowing("mainnet".to_string(), "local-db-remotes".to_string())
        );
    }

    #[test]
    fn test_parse_local_db_remotes_optional_absent_is_ok() {
        // No local-db-remotes key
        let yaml = r#"
test: test
"#;
        let remotes =
            LocalDbRemoteCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        assert!(remotes.is_empty());
    }

    #[test]
    fn test_parse_local_db_remotes_invalid_values() {
        let yaml = r#"
local-db-remotes:
    mainnet:
        - https://example.com/localdb/mainnet
"#;
        let err =
            LocalDbRemoteCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "value".to_string(),
                    expected: "a string".to_string(),
                },
                location: "local-db-remotes[mainnet]".to_string(),
            }
        );

        let yaml = r#"
local-db-remotes:
    mainnet: not_a_valid_url
"#;
        let err =
            LocalDbRemoteCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        match err {
            YamlError::Field { kind, location } => {
                assert_eq!(location, "local-db-remotes[mainnet]".to_string());
                assert!(matches!(kind, FieldErrorKind::InvalidValue { .. }));
            }
            _ => panic!("unexpected error type"),
        }
    }

    #[test]
    fn test_to_yaml_hash() {
        let mut remotes = HashMap::new();
        remotes.insert(
            "mainnet".to_string(),
            LocalDbRemoteCfg {
                document: default_document(),
                key: "mainnet".to_string(),
                url: Url::parse("https://example.com/localdb/mainnet").unwrap(),
            },
        );
        remotes.insert(
            "polygon".to_string(),
            LocalDbRemoteCfg {
                document: default_document(),
                key: "polygon".to_string(),
                url: Url::parse("https://example.com/localdb/polygon").unwrap(),
            },
        );

        let yaml_hash = LocalDbRemoteCfg::to_yaml_hash(&remotes).unwrap();

        let StrictYaml::Hash(hash) = yaml_hash else {
            panic!("expected hash");
        };
        assert_eq!(
            hash.get(&StrictYaml::String("mainnet".to_string())),
            Some(&StrictYaml::String(
                "https://example.com/localdb/mainnet".to_string()
            ))
        );
        assert_eq!(
            hash.get(&StrictYaml::String("polygon".to_string())),
            Some(&StrictYaml::String(
                "https://example.com/localdb/polygon".to_string()
            ))
        );
    }
}
