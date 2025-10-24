use crate::yaml::{
    context::Context, default_document, require_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct LocalDbSyncCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub batch_size: u32,
    pub max_concurrent_batches: u32,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub rate_limit_delay_ms: u64,
    pub finality_depth: u32,
}

impl LocalDbSyncCfg {
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

    fn parse_sync_network_settings(
        network_key: &str,
        yaml: &StrictYaml,
    ) -> Result<LocalDbSyncCfg, YamlError> {
        let location = format!("local-db-sync.{}", network_key);
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

        Ok(LocalDbSyncCfg {
            document: default_document(),
            key: network_key.to_string(),
            batch_size,
            max_concurrent_batches,
            retry_attempts,
            retry_delay_ms,
            rate_limit_delay_ms,
            finality_depth,
        })
    }
}

impl YamlParsableHash for LocalDbSyncCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut syncs: HashMap<String, LocalDbSyncCfg> = HashMap::new();
        let mut seen: HashSet<String> = HashSet::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(sync_hash) = require_hash(
                &document_read,
                Some("local-db-sync"),
                Some("root".to_string()),
            ) {
                for (key_yaml, settings_yaml) in sync_hash {
                    let key = match key_yaml.as_str() {
                        Some(s) if !s.is_empty() => s.to_string(),
                        Some(_) => {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "sync key".to_string(),
                                    reason: "network name cannot be empty".to_string(),
                                },
                                location: "local-db-sync".to_string(),
                            })
                        }
                        None => {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidType {
                                    field: "sync key".to_string(),
                                    expected: "a string".to_string(),
                                },
                                location: "local-db-sync".to_string(),
                            })
                        }
                    };
                    if seen.contains(&key) {
                        return Err(YamlError::KeyShadowing(key, "local-db-sync".to_string()));
                    }
                    seen.insert(key.clone());

                    let mut cfg = LocalDbSyncCfg::parse_sync_network_settings(&key, settings_yaml)?;
                    cfg.document = document.clone();
                    cfg.key = key.clone();
                    syncs.insert(key, cfg);
                }
            }
        }

        if syncs.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("local-db-sync".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(syncs)
    }
}

impl Default for LocalDbSyncCfg {
    fn default() -> Self {
        LocalDbSyncCfg {
            document: default_document(),
            key: String::new(),
            batch_size: 1,
            max_concurrent_batches: 1,
            retry_attempts: 1,
            retry_delay_ms: 1,
            rate_limit_delay_ms: 1,
            finality_depth: 1,
        }
    }
}

impl PartialEq for LocalDbSyncCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.batch_size == other.batch_size
            && self.max_concurrent_batches == other.max_concurrent_batches
            && self.retry_attempts == other.retry_attempts
            && self.retry_delay_ms == other.retry_delay_ms
            && self.rate_limit_delay_ms == other.rate_limit_delay_ms
            && self.finality_depth == other.finality_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    fn full_sync_yaml(
        network: &str,
        batch_size: u32,
        max_concurrent_batches: u32,
        retry_attempts: u32,
        retry_delay_ms: u64,
        rate_limit_delay_ms: u64,
        finality_depth: u32,
    ) -> String {
        format!(
            r#"
local-db-sync:
  {network}:
    batch-size: {batch_size}
    max-concurrent-batches: {max_concurrent_batches}
    retry-attempts: {retry_attempts}
    retry-delay-ms: {retry_delay_ms}
    rate-limit-delay-ms: {rate_limit_delay_ms}
    finality-depth: {finality_depth}
"#
        )
    }

    #[test]
    fn test_parse_sync_missing_section_is_error() {
        let yaml = r#"
not-sync:
  some: value
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("local-db-sync".to_string()),
                location: "root".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_from_yaml_multiple_files() {
        let yaml_one = full_sync_yaml("arbitrum", 100, 5, 3, 50, 10, 100);
        let yaml_two = full_sync_yaml("mainnet", 200, 10, 5, 100, 20, 64);

        let documents = vec![get_document(&yaml_one), get_document(&yaml_two)];
        let syncs = LocalDbSyncCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(syncs.len(), 2);
        assert!(syncs.contains_key("arbitrum"));
        assert!(syncs.contains_key("mainnet"));

        let expected_arb = LocalDbSyncCfg {
            document: default_document(),
            key: "arbitrum".to_string(),
            batch_size: 100,
            max_concurrent_batches: 5,
            retry_attempts: 3,
            retry_delay_ms: 50,
            rate_limit_delay_ms: 10,
            finality_depth: 100,
        };
        let expected_eth = LocalDbSyncCfg {
            document: default_document(),
            key: "mainnet".to_string(),
            batch_size: 200,
            max_concurrent_batches: 10,
            retry_attempts: 5,
            retry_delay_ms: 100,
            rate_limit_delay_ms: 20,
            finality_depth: 64,
        };

        assert_eq!(syncs.get("arbitrum").unwrap(), &expected_arb);
        assert_eq!(syncs.get("mainnet").unwrap(), &expected_eth);
    }

    #[test]
    fn test_parse_sync_from_yaml_duplicate_key() {
        let yaml_one = full_sync_yaml("mainnet", 100, 2, 2, 10, 5, 32);
        let yaml_two = full_sync_yaml("mainnet", 101, 3, 3, 11, 6, 33);

        let documents = vec![get_document(&yaml_one), get_document(&yaml_two)];
        let error = LocalDbSyncCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("mainnet".to_string(), "local-db-sync".to_string())
        );
    }

    #[test]
    fn test_parse_sync_missing_field() {
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    # retry-delay-ms missing
    rate-limit-delay-ms: 1
    finality-depth: 1
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("retry-delay-ms".to_string()),
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_invalid_value_non_numeric() {
        // retry-attempts is non-numeric string, expect invalid value
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: "abc"
    retry-delay-ms: 1
    rate-limit-delay-ms: 1
    finality-depth: 1
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "retry-attempts".to_string(),
                    reason: "invalid digit found in string".to_string()
                },
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_invalid_value_zero_rejected() {
        // rate-limit-delay-ms is zero, which should be rejected as non-positive
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    retry-delay-ms: 1
    rate-limit-delay-ms: 0
    finality-depth: 1
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "rate-limit-delay-ms".to_string(),
                    reason: "must be a positive integer".to_string()
                },
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_empty_key_rejected() {
        // Empty network key should be rejected
        let yaml = r#"
local-db-sync:
  "":
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    retry-delay-ms: 1
    rate-limit-delay-ms: 1
    finality-depth: 1
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "sync key".to_string(),
                    reason: "network name cannot be empty".to_string()
                },
                location: "local-db-sync".to_string(),
            }
        );
    }
}
