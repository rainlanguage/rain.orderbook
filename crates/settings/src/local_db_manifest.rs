use crate::utils::{parse_positive_u32, parse_positive_u64, parse_url};
use crate::yaml::{require_hash, require_string, require_vec, YamlError};
use std::collections::HashMap;
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
    pub address: String,
    pub dump_url: Url,
    pub end_block: u64,
    pub end_block_hash: String,
    pub end_block_time_ms: u64,
}

impl LocalDbManifest {
    pub fn find(&self, chain_id: u64, address: &str) -> Option<&ManifestOrderbook> {
        self.networks
            .values()
            .find(|n| n.chain_id == chain_id)
            .and_then(|n| n.orderbooks.iter().find(|ob| ob.address == address))
    }
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
        let network_key = key_yaml.as_str().unwrap_or_default().to_string();
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
            let _ob_hash = require_hash(ob_yaml, None, Some(location_ob.clone()))?;

            let address = require_string(ob_yaml, Some("address"), Some(location_ob.clone()))?;

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
