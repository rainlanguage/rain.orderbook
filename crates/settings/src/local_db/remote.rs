use crate::yaml::{
    context::Context, default_document, require_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use url::{ParseError as UrlParseError, Url};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LocalDbRemoteCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub url: Url,
}

impl LocalDbRemoteCfg {
    pub fn validate_url(value: &str) -> Result<Url, UrlParseError> {
        Url::parse(value)
    }
}

impl YamlParsableHash for LocalDbRemoteCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut remotes = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(remotes_hash) = require_hash(
                &document_read,
                Some("local-db-remotes"),
                Some("root".to_string()),
            ) {
                for (key_yaml, remote_yaml) in remotes_hash {
                    let key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("local-db-remotes.{}", key);

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
                        key: key.clone(),
                        url,
                    };

                    if remotes.contains_key(&key) {
                        return Err(YamlError::KeyShadowing(key, "local-db-remotes".to_string()));
                    }
                    remotes.insert(key, remote);
                }
            }
        }

        if remotes.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("local-db-remotes".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(remotes)
    }
}

impl Default for LocalDbRemoteCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String(String::new()))),
            key: String::new(),
            url: Url::parse("http://example.com").unwrap(),
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
    use crate::yaml::{tests::get_document, FieldErrorKind};

    #[test]
    fn test_parse_local_db_remotes_missing_section_is_error() {
        let yaml = r#"
not-remotes:
  test: http://example.com
"#;
        let err =
            LocalDbRemoteCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::Missing("local-db-remotes".to_string()),
                location: "root".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_remotes_invalid_url() {
        let yaml = r#"
local-db-remotes:
  test: test
"#;
        let err =
            LocalDbRemoteCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "url".to_string(),
                    reason: "relative URL without a base".to_string(),
                },
                location: "local-db-remotes.test".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_remotes_duplicate_key_across_files() {
        let yaml = r#"
local-db-remotes:
  test: http://example.com
"#;
        let err = LocalDbRemoteCfg::parse_all_from_yaml(
            vec![get_document(yaml), get_document(yaml)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            YamlError::KeyShadowing("test".to_string(), "local-db-remotes".to_string())
        );
    }

    #[test]
    fn test_parse_local_db_remotes_success() {
        let yaml = r#"
local-db-remotes:
  primary: https://example.com
  secondary: https://rainlang.xyz
"#;
        let remotes =
            LocalDbRemoteCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();

        assert_eq!(remotes.len(), 2);

        let primary = remotes.get("primary").unwrap();
        assert_eq!(primary.key, "primary");
        assert_eq!(primary.url, Url::parse("https://example.com").unwrap());

        let secondary = remotes.get("secondary").unwrap();
        assert_eq!(secondary.key, "secondary");
        assert_eq!(secondary.url, Url::parse("https://rainlang.xyz").unwrap());
    }
}
