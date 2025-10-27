use crate::utils::{parse_positive_u32, parse_positive_u64, parse_url};
use crate::yaml::{require_hash, require_string, require_vec, FieldErrorKind, YamlError};
use alloy::primitives::Address;
use std::collections::HashMap;
use std::str::FromStr;
use strict_yaml_rust::StrictYaml;
use url::Url;

pub const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDbManifest {
    pub manifest_version: u32,
    pub db_schema_version: u32,
    pub networks: HashMap<String, ManifestNetwork>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestNetwork {
    pub chain_id: u64,
    pub orderbooks: Vec<ManifestOrderbook>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestOrderbook {
    pub address: Address,
    pub dump_url: Url,
    pub end_block: u64,
    pub end_block_hash: String,
    pub end_block_time_ms: u64,
}

impl LocalDbManifest {
    pub fn find(&self, chain_id: u64, address: Address) -> Option<&ManifestOrderbook> {
        self.networks
            .values()
            .find(|n| n.chain_id == chain_id)
            .and_then(|n| n.orderbooks.iter().find(|ob| ob.address == address))
    }
}

pub fn current_manifest_version() -> u32 {
    MANIFEST_VERSION
}

pub fn is_manifest_version_current(version: u32) -> bool {
    version == MANIFEST_VERSION
}

pub fn parse_manifest_doc(doc: &StrictYaml) -> Result<LocalDbManifest, YamlError> {
    let location_root = "manifest".to_string();

    let manifest_version = parse_positive_u32(
        &require_string(doc, Some("manifest-version"), Some(location_root.clone()))?,
        "manifest-version",
        location_root.clone(),
    )?;

    let db_schema_version = parse_positive_u32(
        &require_string(doc, Some("db-schema-version"), Some(location_root.clone()))?,
        "db-schema-version",
        location_root.clone(),
    )?;

    let networks_hash = require_hash(doc, Some("networks"), Some(location_root.clone()))?;
    let mut networks: HashMap<String, ManifestNetwork> = HashMap::new();

    for (key_yaml, network_yaml) in networks_hash.iter() {
        let network_key = key_yaml
            .as_str()
            .ok_or(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "key".to_string(),
                    expected: "a string".to_string(),
                },
                location: "manifest.networks".to_string(),
            })?
            .to_string();
        let location_network = format!("manifest.networks.{}", network_key);

        let _network_hash = require_hash(network_yaml, None, Some(location_network.clone()))?;

        let chain_id = parse_positive_u64(
            &require_string(
                network_yaml,
                Some("chain-id"),
                Some(location_network.clone()),
            )?,
            "chain-id",
            location_network.clone(),
        )?;

        let orderbooks_yaml =
            require_vec(network_yaml, "orderbooks", Some(location_network.clone()))?;

        let mut orderbooks: Vec<ManifestOrderbook> = Vec::new();
        for (idx, ob_yaml) in orderbooks_yaml.iter().enumerate() {
            let location_ob = format!("{}.orderbooks[{}]", location_network, idx);

            let address_str = require_string(ob_yaml, Some("address"), Some(location_ob.clone()))?;
            let address = Address::from_str(&address_str).map_err(|e| YamlError::Field {
                kind: crate::yaml::FieldErrorKind::InvalidValue {
                    field: "address".to_string(),
                    reason: e.to_string(),
                },
                location: location_ob.clone(),
            })?;

            let dump_url_str =
                require_string(ob_yaml, Some("dump-url"), Some(location_ob.clone()))?;
            let dump_url = parse_url(&dump_url_str, "dump-url", location_ob.clone())?;

            let end_block = parse_positive_u64(
                &require_string(ob_yaml, Some("end-block"), Some(location_ob.clone()))?,
                "end-block",
                location_ob.clone(),
            )?;

            let end_block_hash =
                require_string(ob_yaml, Some("end-block-hash"), Some(location_ob.clone()))?;

            let end_block_time_ms = parse_positive_u64(
                &require_string(
                    ob_yaml,
                    Some("end-block-time-ms"),
                    Some(location_ob.clone()),
                )?,
                "end-block-time-ms",
                location_ob.clone(),
            )?;

            orderbooks.push(ManifestOrderbook {
                address,
                dump_url,
                end_block,
                end_block_hash,
                end_block_time_ms,
            });
        }

        networks.insert(
            network_key.clone(),
            ManifestNetwork {
                chain_id,
                orderbooks,
            },
        );
    }

    Ok(LocalDbManifest {
        manifest_version,
        db_schema_version,
        networks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use strict_yaml_rust::StrictYamlLoader;

    fn load(yaml: &str) -> StrictYaml {
        StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone()
    }

    #[test]
    fn test_missing_manifest_version() {
        let yaml = r#"
db-schema-version: 1
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_missing_db_schema_version() {
        let yaml = r#"
manifest-version: 1
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_missing_networks() {
        let yaml = r#"
manifest-version: 1
db-schema-version: 1
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_zero_or_invalid_manifest_and_schema_versions() {
        let yaml_zero_manifest = r#"
manifest-version: 0
db-schema-version: 1
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml_zero_manifest)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let yaml_zero_schema = r#"
manifest-version: 1
db-schema-version: 0
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml_zero_schema)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_network_missing_chain_id_and_invalid() {
        let yaml_missing = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet: {}
"#;
        let err = parse_manifest_doc(&load(yaml_missing)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let yaml_zero = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 0
    orderbooks: []
"#;
        let err = parse_manifest_doc(&load(yaml_zero)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_orderbooks_required_and_type() {
        let yaml_missing = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
"#;
        let err = parse_manifest_doc(&load(yaml_missing)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let yaml_non_list = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks: {}
"#;
        let err = parse_manifest_doc(&load(yaml_non_list)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_orderbook_missing_fields() {
        // Full, valid baseline
        let good = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block: 1
        end-block-hash: "0xabc"
        end-block-time-ms: 1
"#;
        assert!(parse_manifest_doc(&load(good)).is_ok());

        // Now omit each required field individually
        let missing_address = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - dump-url: "http://example.com"
        end-block: 1
        end-block-hash: "0xabc"
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_address)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_dump = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x0000000000000000000000000000000000000001"
        end-block: 1
        end-block-hash: "0xabc"
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_dump)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_end_block = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block-hash: "0xabc"
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_end_block)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_end_hash = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block: 1
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_end_hash)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_end_time = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block: 1
        end-block-hash: "0xabc"
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_end_time)).unwrap_err(),
            YamlError::Field { .. }
        ));
    }

    #[test]
    fn test_find_across_networks_and_negatives() {
        let yaml = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x1111111111111111111111111111111111111111"
        dump-url: "http://example.com/a"
        end-block: 10
        end-block-hash: "0xa"
        end-block-time-ms: 100
  other:
    chain-id: 2
    orderbooks:
      - address: "0x2222222222222222222222222222222222222222"
        dump-url: "http://example.com/b"
        end-block: 20
        end-block-hash: "0xb"
        end-block-time-ms: 200
"#;
        let m = parse_manifest_doc(&load(yaml)).unwrap();

        assert!(m
            .find(
                1,
                Address::from_str("0x1111111111111111111111111111111111111111").unwrap()
            )
            .is_some());
        assert!(m
            .find(
                2,
                Address::from_str("0x2222222222222222222222222222222222222222").unwrap()
            )
            .is_some());

        assert!(m
            .find(
                1,
                Address::from_str("0xdeadbeef00000000000000000000000000000000").unwrap()
            )
            .is_none());
        assert!(m
            .find(
                999,
                Address::from_str("0x1111111111111111111111111111111111111111").unwrap()
            )
            .is_none());
    }

    #[test]
    fn test_manifest_version_helpers() {
        assert!(is_manifest_version_current(current_manifest_version()));
        assert!(!is_manifest_version_current(0));
    }
}
