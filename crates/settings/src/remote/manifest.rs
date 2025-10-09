use crate::yaml::{get_hash_value, load_yaml, require_hash, require_string, YamlError};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::{collections::BTreeMap, num::ParseIntError, time::Duration};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::Url;

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub schema_version: u32,
    pub chains: BTreeMap<u32, ManifestEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestEntry {
    pub dump_url: Url,
    pub dump_timestamp: DateTime<Utc>,
    pub seed_generation: u32,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("HTTP error fetching manifest: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Failed to parse manifest YAML: {0}")]
    Parse(String),
    #[error("Manifest YAML is empty")]
    EmptyDocument,
    #[error("Manifest schema version {found} does not match expected {expected}")]
    SchemaMismatch { expected: u32, found: u32 },
    #[error("Manifest does not contain any chains")]
    MissingChains,
    #[error("Invalid chain id '{key}' in manifest: {source}")]
    InvalidChainId {
        key: String,
        #[source]
        source: ParseIntError,
    },
    #[error("Invalid dump timestamp for chain {chain_id}: {source}")]
    InvalidTimestamp {
        chain_id: u32,
        #[source]
        source: chrono::ParseError,
    },
    #[error("Chain {chain_id} not found in manifest")]
    ChainNotFound { chain_id: u32 },
    #[error("YAML error: {0}")]
    Yaml(#[from] YamlError),
}

impl Manifest {
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn chains(&self) -> &BTreeMap<u32, ManifestEntry> {
        &self.chains
    }

    pub fn entry(&self, chain_id: u32) -> Result<&ManifestEntry, ManifestError> {
        self.chains
            .get(&chain_id)
            .ok_or(ManifestError::ChainNotFound { chain_id })
    }

    pub fn dump_url(&self, chain_id: u32) -> Result<&Url, ManifestError> {
        let entry = self.entry(chain_id)?;
        Ok(&entry.dump_url)
    }

    pub fn dump_timestamp(&self, chain_id: u32) -> Result<DateTime<Utc>, ManifestError> {
        let entry = self.entry(chain_id)?;
        Ok(entry.dump_timestamp)
    }

    pub fn seed_generation(&self, chain_id: u32) -> Result<u32, ManifestError> {
        let entry = self.entry(chain_id)?;
        Ok(entry.seed_generation)
    }

    pub fn parse_from_str(yaml: &str) -> Result<Self, ManifestError> {
        let document = load_yaml(yaml)?;
        Self::try_from_yaml(document)
    }

    pub async fn fetch(client: &reqwest::Client, url: &Url) -> Result<Self, ManifestError> {
        let response = client.get(url.clone()).send().await?.error_for_status()?;
        let body = response.text().await?;
        Self::parse_from_str(&body)
    }

    fn try_from_yaml(document: StrictYaml) -> Result<Self, ManifestError> {
        let root = require_hash(&document, None, Some("manifest".to_string()))?;

        let schema_version_yaml =
            get_hash_value(root, "schema_version", Some("manifest".to_string()))?;
        let schema_version = require_string(schema_version_yaml, None, None)?;
        let schema_version = schema_version.parse::<u32>().map_err(|_| {
            ManifestError::Parse("Manifest field 'schema_version' must be an integer".to_string())
        })?;

        if schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ManifestError::SchemaMismatch {
                expected: MANIFEST_SCHEMA_VERSION,
                found: schema_version,
            });
        }

        let chains_node = get_hash_value(root, "chains", Some("manifest".to_string()))?;
        let chains_hash = chains_node.as_hash().ok_or_else(|| {
            ManifestError::Parse("Manifest field 'chains' must be a map".to_string())
        })?;
        if chains_hash.is_empty() {
            return Err(ManifestError::MissingChains);
        }

        let mut chains = BTreeMap::new();

        for (key_node, value_node) in chains_hash {
            let key_str = key_node.as_str().ok_or_else(|| {
                ManifestError::Parse("Chain ids must be strings in manifest".to_string())
            })?;
            let key_string = key_str.to_string();
            let chain_id =
                key_string
                    .parse::<u32>()
                    .map_err(|source| ManifestError::InvalidChainId {
                        key: key_string.clone(),
                        source,
                    })?;
            let location = format!("chain {}", chain_id);

            let entry_hash = require_hash(value_node, None, Some(location.clone()))?;

            let dump_url_str =
                require_string(value_node, Some("dump_url"), Some(location.clone()))?;
            let dump_url = Url::parse(&dump_url_str).map_err(|err| {
                ManifestError::Parse(format!("Invalid dump_url for chain {chain_id}: {err}"))
            })?;

            let dump_timestamp_str =
                require_string(value_node, Some("dump_timestamp"), Some(location.clone()))?;
            let dump_timestamp = DateTime::parse_from_rfc3339(&dump_timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|source| ManifestError::InvalidTimestamp { chain_id, source })?;

            let seed_generation_node =
                get_hash_value(entry_hash, "seed_generation", Some(location.clone()))?;
            let seed_generation =
                require_string(seed_generation_node, None, Some(location.clone()))?;
            let seed_generation = seed_generation.parse::<u32>().map_err(|_| {
                ManifestError::Parse(format!(
                    "Chain {chain_id} field 'seed_generation' must be an integer"
                ))
            })?;

            chains.insert(
                chain_id,
                ManifestEntry {
                    dump_url,
                    dump_timestamp,
                    seed_generation,
                },
            );
        }

        if chains.is_empty() {
            return Err(ManifestError::MissingChains);
        }

        Ok(Self {
            schema_version,
            chains,
        })
    }
}

impl ManifestEntry {
    pub fn is_stale(&self, max_age: Duration, now: DateTime<Utc>) -> bool {
        match ChronoDuration::from_std(max_age) {
            Ok(threshold) => now - self.dump_timestamp > threshold,
            Err(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use httpmock::prelude::*;

    #[test]
    fn parses_manifest_from_yaml() {
        let yaml = r#"
schema_version: 1
chains:
  42161:
    dump_url: https://example.com/arbitrum.sql.gz
    dump_timestamp: "2024-04-30T12:34:56Z"
    seed_generation: 3
  8453:
    dump_url: https://example.com/base.sql.gz
    dump_timestamp: "2024-05-01T00:00:00Z"
    seed_generation: 4
"#;

        let manifest = Manifest::parse_from_str(yaml).expect("manifest parses");
        assert_eq!(manifest.schema_version(), 1);
        assert!(manifest.entry(42161).is_ok());
        assert!(manifest.entry(8453).is_ok());

        let entry = manifest.entry(42161).unwrap().clone();
        assert_eq!(
            entry.dump_url,
            Url::parse("https://example.com/arbitrum.sql.gz").unwrap()
        );
        assert_eq!(
            entry.dump_timestamp,
            Utc.with_ymd_and_hms(2024, 4, 30, 12, 34, 56).unwrap()
        );
        assert_eq!(entry.seed_generation, 3);
    }

    #[test]
    fn schema_version_mismatch_errors() {
        let yaml = r#"
schema_version: 999
chains:
  1:
    dump_url: https://example.com.sql
    dump_timestamp: "2024-05-01T00:00:00Z"
    seed_generation: 1
"#;

        let err = Manifest::parse_from_str(yaml).unwrap_err();
        match err {
            ManifestError::SchemaMismatch { expected, found } => {
                assert_eq!(expected, MANIFEST_SCHEMA_VERSION);
                assert_eq!(found, 999);
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn invalid_chain_id_errors() {
        let yaml = r#"
schema_version: 1
chains:
  banana:
    dump_url: https://example.com.sql
    dump_timestamp: "2024-05-01T00:00:00Z"
    seed_generation: 1
"#;

        let err = Manifest::parse_from_str(yaml).unwrap_err();
        matches!(err, ManifestError::InvalidChainId { .. });
    }

    #[test]
    fn stale_detection_uses_timestamp() {
        let entry = ManifestEntry {
            dump_url: Url::parse("https://example.com").unwrap(),
            dump_timestamp: Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap(),
            seed_generation: 1,
        };
        let now = Utc.with_ymd_and_hms(2024, 5, 2, 0, 0, 0).unwrap();

        assert!(entry.is_stale(Duration::from_secs(0), now));
        assert!(!entry.is_stale(Duration::from_secs(86_400), now));
        assert!(entry.is_stale(Duration::from_secs(10), now));
    }

    #[tokio::test]
    async fn fetches_manifest_over_http() {
        let server = MockServer::start_async().await;
        let yaml = r#"
schema_version: 1
chains:
  1:
    dump_url: https://example.com/dump.sql.gz
    dump_timestamp: "2024-05-01T00:00:00Z"
    seed_generation: 42
"#;

        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/manifest.yaml");
                then.status(200).body(yaml);
            })
            .await;

        let client = reqwest::Client::new();
        let url = Url::parse(&server.url("/manifest.yaml")).unwrap();

        let manifest = Manifest::fetch(&client, &url)
            .await
            .expect("fetch succeeds");
        let entry = manifest.entry(1).expect("entry exists");

        assert_eq!(
            entry.dump_url,
            Url::parse("https://example.com/dump.sql.gz").unwrap()
        );
        assert_eq!(
            entry.dump_timestamp,
            Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap()
        );
        assert_eq!(entry.seed_generation, 42);
    }

    #[tokio::test]
    async fn fetch_propagates_http_errors() {
        let server = MockServer::start_async().await;
        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/manifest.yaml");
                then.status(500).body("server error");
            })
            .await;

        let client = reqwest::Client::new();
        let url = Url::parse(&server.url("/manifest.yaml")).unwrap();

        let err = Manifest::fetch(&client, &url)
            .await
            .expect_err("fetch fails");
        assert!(matches!(err, ManifestError::Http(_)));
    }

    #[tokio::test]
    async fn fetch_surfaces_parse_error_for_malformed_yaml() {
        let server = MockServer::start_async().await;
        let malformed_yaml = r#"
schema_version: 1
# missing chains section entirely
"#;

        let _mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/manifest.yaml");
                then.status(200).body(malformed_yaml);
            })
            .await;

        let client = reqwest::Client::new();
        let url = Url::parse(&server.url("/manifest.yaml")).unwrap();

        let err = Manifest::fetch(&client, &url)
            .await
            .expect_err("fetch fails");
        assert!(matches!(err, ManifestError::MissingChains));
    }
}
