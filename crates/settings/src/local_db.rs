use crate::yaml::context::Context;
use crate::yaml::{
    default_document, get_hash_value_as_option, optional_hash, require_hash, require_string,
    FieldErrorKind, YamlError, YamlParseableValue,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::strict_yaml::Hash as YamlHash;
use strict_yaml_rust::StrictYaml;
use url::{ParseError as UrlParseError, Url};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SyncSettings {
    pub batch_size: u32,
    pub max_concurrent_batches: u32,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub rate_limit_delay_ms: u64,
    pub finality_depth: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LocalDbCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub manifest_url: Url,
    pub sync: HashMap<String, SyncSettings>,
}

impl LocalDbCfg {
    fn parse_positive_u32(value: &str, field: &str, location: String) -> Result<u32, YamlError> {
        let parsed: u32 = value
            .parse()
            .map_err(|e: std::num::ParseIntError| YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: field.to_string(),
                    reason: e.to_string(),
                },
                location: location.clone(),
            })?;
        if parsed == 0 {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: field.to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location,
            });
        }
        Ok(parsed)
    }

    fn parse_positive_u64(value: &str, field: &str, location: String) -> Result<u64, YamlError> {
        let parsed: u64 = value
            .parse()
            .map_err(|e: std::num::ParseIntError| YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: field.to_string(),
                    reason: e.to_string(),
                },
                location: location.clone(),
            })?;
        if parsed == 0 {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: field.to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location,
            });
        }
        Ok(parsed)
    }

    fn parse_manifest_url(value: &str) -> Result<Url, UrlParseError> {
        Url::parse(value)
    }

    fn parse_sync_network_settings(
        network_key: &str,
        yaml: &StrictYaml,
    ) -> Result<SyncSettings, YamlError> {
        let location = format!("local-db.sync.{}", network_key);
        let batch_size = Self::parse_positive_u32(
            &require_string(yaml, Some("batch-size"), Some(location.clone()))?,
            "batch-size",
            location.clone(),
        )?;
        let max_concurrent_batches = Self::parse_positive_u32(
            &require_string(yaml, Some("max-concurrent-batches"), Some(location.clone()))?,
            "max-concurrent-batches",
            location.clone(),
        )?;
        let retry_attempts = Self::parse_positive_u32(
            &require_string(yaml, Some("retry-attempts"), Some(location.clone()))?,
            "retry-attempts",
            location.clone(),
        )?;
        let retry_delay_ms = Self::parse_positive_u64(
            &require_string(yaml, Some("retry-delay-ms"), Some(location.clone()))?,
            "retry-delay-ms",
            location.clone(),
        )?;
        let rate_limit_delay_ms = Self::parse_positive_u64(
            &require_string(yaml, Some("rate-limit-delay-ms"), Some(location.clone()))?,
            "rate-limit-delay-ms",
            location.clone(),
        )?;
        let finality_depth = Self::parse_positive_u32(
            &require_string(yaml, Some("finality-depth"), Some(location.clone()))?,
            "finality-depth",
            location,
        )?;

        Ok(SyncSettings {
            batch_size,
            max_concurrent_batches,
            retry_attempts,
            retry_delay_ms,
            rate_limit_delay_ms,
            finality_depth,
        })
    }
}

impl YamlParseableValue for LocalDbCfg {
    fn parse_from_yaml(
        _: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<LocalDbCfg, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }

    fn parse_from_yaml_optional(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<Option<LocalDbCfg>, YamlError> {
        let mut found_local_db = false;
        let mut manifest_url: Option<Url> = None;
        let mut manifest_url_str: Option<String> = None;
        let mut sync: HashMap<String, SyncSettings> = HashMap::new();
        let mut seen_networks: HashSet<String> = HashSet::new();
        let mut document_index: usize = 0;

        for (index, document) in documents.iter().enumerate() {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(local_db_hash) = optional_hash(&document_read, "local-db") {
                found_local_db = true;
                document_index = index;

                // manifest-url is required overall when local-db is present
                if let Some(manifest_yaml) = get_hash_value_as_option(local_db_hash, "manifest-url")
                {
                    let s = require_string(manifest_yaml, None, Some("local-db".to_string()))?;
                    let parsed =
                        LocalDbCfg::parse_manifest_url(&s).map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "manifest-url".to_string(),
                                reason: e.to_string(),
                            },
                            location: "local-db".to_string(),
                        })?;

                    if let Some(existing) = &manifest_url_str {
                        if existing != &s {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "manifest-url".to_string(),
                                    reason: "conflicting values across documents".to_string(),
                                },
                                location: "local-db".to_string(),
                            });
                        }
                    } else {
                        manifest_url = Some(parsed);
                        manifest_url_str = Some(s);
                    }
                }

                if let Some(sync_yaml) = get_hash_value_as_option(local_db_hash, "sync") {
                    let sync_hash: &YamlHash =
                        require_hash(sync_yaml, None, Some("local-db".to_string()))?;

                    for (key_yaml, settings_yaml) in sync_hash.iter() {
                        let network_key = match key_yaml.as_str() {
                            Some(s) if !s.is_empty() => s.to_string(),
                            Some(_) => {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "sync key".to_string(),
                                        reason: "network name cannot be empty".to_string(),
                                    },
                                    location: "local-db.sync".to_string(),
                                })
                            }
                            None => {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidType {
                                        field: "sync key".to_string(),
                                        expected: "a string".to_string(),
                                    },
                                    location: "local-db.sync".to_string(),
                                })
                            }
                        };
                        if seen_networks.contains(&network_key) {
                            return Err(YamlError::KeyShadowing(
                                network_key,
                                "local-db.sync".to_string(),
                            ));
                        }
                        seen_networks.insert(network_key.clone());

                        let settings =
                            LocalDbCfg::parse_sync_network_settings(&network_key, settings_yaml)?;
                        sync.insert(network_key, settings);
                    }
                }
            }
        }

        if !found_local_db {
            return Ok(None);
        }

        let manifest_url = match manifest_url {
            Some(url) => url,
            None => {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("manifest-url".to_string()),
                    location: "local-db".to_string(),
                });
            }
        };

        Ok(Some(LocalDbCfg {
            document: documents[document_index].clone(),
            manifest_url,
            sync,
        }))
    }
}

impl Default for LocalDbCfg {
    fn default() -> Self {
        LocalDbCfg {
            document: default_document(),
            manifest_url: Url::parse("http://example.com").unwrap(),
            sync: HashMap::new(),
        }
    }
}

impl PartialEq for LocalDbCfg {
    fn eq(&self, other: &Self) -> bool {
        self.manifest_url == other.manifest_url && self.sync == other.sync
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_local_db_full_config() {
        let yaml = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let cfg = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None)
            .unwrap()
            .unwrap();
        assert_eq!(
            cfg.manifest_url.to_string(),
            "https://example.com/manifest.json"
        );
        assert!(cfg.sync.contains_key("mainnet"));
        let s = cfg.sync.get("mainnet").unwrap();
        assert_eq!(s.batch_size, 2000);
        assert_eq!(s.max_concurrent_batches, 4);
        assert_eq!(s.retry_attempts, 3);
        assert_eq!(s.retry_delay_ms, 1000);
        assert_eq!(s.rate_limit_delay_ms, 1);
        assert_eq!(s.finality_depth, 64);
    }

    #[test]
    fn test_parse_local_db_missing_manifest_url() {
        let yaml = r#"
local-db:
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let err = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::Missing("manifest-url".to_string()),
                location: "local-db".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_conflicting_manifest_url() {
        let yaml1 = r#"
local-db:
  manifest-url: https://example.com/one.json
"#;
        let yaml2 = r#"
local-db:
  manifest-url: https://example.com/two.json
"#;
        let err = LocalDbCfg::parse_from_yaml_optional(
            vec![get_document(yaml1), get_document(yaml2)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "manifest-url".to_string(),
                    reason: "conflicting values across documents".to_string(),
                },
                location: "local-db".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_duplicate_sync_keys() {
        let yaml1 = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let yaml2 = r#"
local-db:
  sync:
    mainnet:
      batch-size: 1000
      max-concurrent-batches: 2
      retry-attempts: 3
      retry-delay-ms: 500
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let err = LocalDbCfg::parse_from_yaml_optional(
            vec![get_document(yaml1), get_document(yaml2)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            YamlError::KeyShadowing("mainnet".to_string(), "local-db.sync".to_string())
        );
    }

    #[test]
    fn test_parse_local_db_missing_sync_field() {
        let yaml = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      # finality-depth missing
"#;
        let err = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::Missing("finality-depth".to_string()),
                location: "local-db.sync.mainnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_zero_or_negative_values() {
        let yaml_zero = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 0
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let err =
            LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml_zero)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "batch-size".to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location: "local-db.sync.mainnet".to_string(),
            }
        );

        let yaml_negative = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: -1
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        // Negative numbers will fail to parse as u32 and surface as InvalidValue with parse error
        let err = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml_negative)], None)
            .unwrap_err();
        match err {
            YamlError::Field { kind, location } => {
                assert_eq!(location, "local-db.sync.mainnet".to_string());
                match kind {
                    FieldErrorKind::InvalidValue { field, .. } => {
                        assert_eq!(field, "batch-size".to_string());
                    }
                    _ => panic!("unexpected error kind"),
                }
            }
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_parse_from_yaml_invalid_trait() {
        let yaml = "foo: bar";
        let err = LocalDbCfg::parse_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(err, YamlError::InvalidTraitFunction);
    }

    #[test]
    fn test_parse_local_db_absent_returns_none() {
        let yaml = "some: value";
        let res = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_parse_local_db_invalid_manifest_url() {
        let yaml = r#"
local-db:
  manifest-url: not-a-valid-url
"#;
        let err = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        match err {
            YamlError::Field { kind, location } => {
                assert_eq!(location, "local-db".to_string());
                match kind {
                    FieldErrorKind::InvalidValue { field, .. } => {
                        assert_eq!(field, "manifest-url".to_string());
                    }
                    _ => panic!("unexpected error kind"),
                }
            }
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_parse_local_db_zero_values_u64_fields() {
        let yaml = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 0
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let err = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "retry-delay-ms".to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location: "local-db.sync.mainnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_negative_values_u64_fields() {
        let yaml = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: -5
      finality-depth: 64
"#;
        // Negative u64 should surface as parse error on the field
        let err = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        match err {
            YamlError::Field { kind, location } => {
                assert_eq!(location, "local-db.sync.mainnet".to_string());
                match kind {
                    FieldErrorKind::InvalidValue { field, .. } => {
                        assert_eq!(field, "rate-limit-delay-ms".to_string());
                    }
                    _ => panic!("unexpected error kind"),
                }
            }
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_parse_local_db_zero_rate_limit_delay_ms() {
        let yaml = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 0
      finality-depth: 64
"#;
        let err = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "rate-limit-delay-ms".to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location: "local-db.sync.mainnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_zero_values_other_u32_fields() {
        // max-concurrent-batches = 0
        let yaml_mcb = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 0
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let err =
            LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml_mcb)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "max-concurrent-batches".to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location: "local-db.sync.mainnet".to_string(),
            }
        );

        // retry-attempts = 0
        let yaml_ra = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 0
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let err =
            LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml_ra)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "retry-attempts".to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location: "local-db.sync.mainnet".to_string(),
            }
        );

        // finality-depth = 0
        let yaml_fd = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 0
"#;
        let err =
            LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml_fd)], None).unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "finality-depth".to_string(),
                    reason: "must be a positive integer".to_string(),
                },
                location: "local-db.sync.mainnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_local_db_merge_sync_from_multiple_documents() {
        let yaml1 = r#"
local-db:
  manifest-url: https://example.com/manifest.json
  sync:
    mainnet:
      batch-size: 2000
      max-concurrent-batches: 4
      retry-attempts: 3
      retry-delay-ms: 1000
      rate-limit-delay-ms: 1
      finality-depth: 64
"#;
        let yaml2 = r#"
local-db:
  sync:
    goerli:
      batch-size: 1500
      max-concurrent-batches: 2
      retry-attempts: 5
      retry-delay-ms: 2000
      rate-limit-delay-ms: 1
      finality-depth: 32
"#;
        let cfg = LocalDbCfg::parse_from_yaml_optional(
            vec![get_document(yaml1), get_document(yaml2)],
            None,
        )
        .unwrap()
        .unwrap();

        assert!(cfg.sync.contains_key("mainnet"));
        assert!(cfg.sync.contains_key("goerli"));
        assert_eq!(cfg.sync.get("mainnet").unwrap().finality_depth, 64);
        assert_eq!(cfg.sync.get("goerli").unwrap().finality_depth, 32);
    }

    #[test]
    fn test_parse_local_db_without_sync_is_ok_with_empty_map() {
        let yaml = r#"
local-db:
  manifest-url: https://example.com/manifest.json
"#;
        let cfg = LocalDbCfg::parse_from_yaml_optional(vec![get_document(yaml)], None)
            .unwrap()
            .unwrap();
        assert!(cfg.sync.is_empty());
    }

    #[test]
    fn test_default_and_partial_eq() {
        // Default equality
        let a = LocalDbCfg::default();
        let b = LocalDbCfg::default();
        assert_eq!(a, b);

        // Same logical fields, different document Arc; should still be equal
        let c = LocalDbCfg {
            document: get_document("x: 1"),
            manifest_url: Url::parse("http://example.com").unwrap(),
            sync: std::collections::HashMap::new(),
        };
        assert_eq!(a, c);
    }
}
