use super::ProducerOutcome;
use alloy::primitives::{Address, Bytes};
use rain_orderbook_app_settings::local_db_manifest::{LocalDbManifest, ManifestOrderbook};
use rain_orderbook_common::local_db::pipeline::runner::utils::RunnerTarget;
use rain_orderbook_common::local_db::{LocalDbError, OrderbookIdentifier};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use tokio::fs;
use url::Url;

/// Builds a manifest from successful producer outcomes.
pub fn build_manifest(
    successes: &[ProducerOutcome],
    target_lookup: &HashMap<OrderbookIdentifier, RunnerTarget>,
    release_base_url: &Url,
) -> Result<LocalDbManifest, LocalDbError> {
    let mut per_network: HashMap<String, (u32, Vec<ManifestOrderbook>)> = HashMap::new();

    for outcome in successes {
        let export = match &outcome.exported_dump {
            Some(export) => export,
            None => continue,
        };
        let ob_id = &outcome.outcome.ob_id;
        let runner_target =
            target_lookup
                .get(ob_id)
                .ok_or_else(|| LocalDbError::MissingRunnerTarget {
                    chain_id: ob_id.chain_id,
                    orderbook_address: ob_id.orderbook_address,
                })?;

        let chain_id = ob_id.chain_id;
        let network_key = runner_target.network_key.clone();
        let dump_url = build_dump_url(release_base_url, chain_id, ob_id.orderbook_address)?;
        let end_block_hash = Bytes::from_str(export.end_block_hash.as_str())?;

        let manifest_orderbook = ManifestOrderbook {
            address: ob_id.orderbook_address,
            dump_url,
            end_block: export.end_block,
            end_block_hash,
            end_block_time_ms: export.end_block_time_ms,
        };

        let entry = per_network
            .entry(network_key.clone())
            .or_insert_with(|| (chain_id, Vec::new()));
        if entry.0 != chain_id {
            return Err(LocalDbError::RunnerNetworkChainIdMismatch {
                network_key,
                expected: entry.0,
                found: chain_id,
            });
        }
        entry.1.push(manifest_orderbook);
    }

    let mut manifest = LocalDbManifest::new();
    let mut network_keys: Vec<_> = per_network.keys().cloned().collect();
    network_keys.sort();

    for key in network_keys {
        if let Some((chain_id, mut orderbooks)) = per_network.remove(&key) {
            orderbooks.sort_by(|a, b| a.address.cmp(&b.address));
            manifest.add_network(&key, chain_id)?;
            for orderbook in orderbooks {
                manifest.push_orderbook(&key, orderbook)?;
            }
        }
    }

    Ok(manifest)
}

fn build_dump_url(
    base_url: &Url,
    chain_id: u32,
    orderbook_address: Address,
) -> Result<Url, LocalDbError> {
    let base = base_url.as_str().trim_end_matches('/');
    let address_str = orderbook_address.to_string();
    let url_str = format!("{}/{}-{}.sql.gz", base, chain_id, address_str);
    Url::parse(&url_str).map_err(|source| LocalDbError::DumpUrlConstructionFailed {
        url: url_str,
        source,
    })
}

/// Writes the manifest to disk, creating parent directories if necessary.
pub async fn write_manifest_to_path(
    manifest: &LocalDbManifest,
    path: &Path,
) -> Result<(), LocalDbError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let yaml = manifest.to_yaml_string()?;
    fs::write(path, yaml).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::local_db::pipeline::runner::export::ExportMetadata;
    use crate::commands::local_db::pipeline::runner::ProducerOutcome;
    use alloy::primitives::address;
    use rain_orderbook_common::local_db::pipeline::engine::SyncInputs;
    use rain_orderbook_common::local_db::pipeline::{
        FinalityConfig, SyncConfig, SyncOutcome, WindowOverrides,
    };
    use rain_orderbook_common::local_db::{FetchConfig, OrderbookIdentifier};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn sample_runner_target(network_key: &str, chain_id: u32, address: Address) -> RunnerTarget {
        let fetch = FetchConfig::new(10, 5, 5, 1).unwrap();
        RunnerTarget {
            orderbook_key: format!("{}-{}", network_key, address),
            manifest_url: Url::parse("https://example.com/manifest.yaml").unwrap(),
            network_key: network_key.to_string(),
            inputs: SyncInputs {
                ob_id: OrderbookIdentifier {
                    chain_id,
                    orderbook_address: address,
                },
                metadata_rpcs: Vec::new(),
                cfg: SyncConfig {
                    deployment_block: 0,
                    fetch,
                    finality: FinalityConfig { depth: 12 },
                    window_overrides: WindowOverrides::default(),
                },
                dump_str: None,
                block_number_threshold: 10000,
                manifest_end_block: 1,
            },
        }
    }

    fn sample_outcome(target: &OrderbookIdentifier, dump_suffix: &str) -> ProducerOutcome {
        ProducerOutcome {
            outcome: SyncOutcome {
                ob_id: target.clone(),
                start_block: 0,
                target_block: 1234,
                fetched_logs: 10,
                decoded_events: 5,
            },
            exported_dump: Some(ExportMetadata {
                dump_path: Path::new(dump_suffix).to_path_buf(),
                end_block: 1234,
                end_block_hash: "0xfeedface".to_string(),
                end_block_time_ms: 1_700_000_000,
            }),
        }
    }

    #[test]
    fn build_manifest_skips_missing_exported_dump() {
        let target_included = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000aa1"),
        };
        let target_skipped = OrderbookIdentifier {
            chain_id: 10,
            orderbook_address: address!("0x0000000000000000000000000000000000000bb2"),
        };

        let mut lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        lookup.insert(
            target_included.clone(),
            sample_runner_target(
                "arbitrum",
                target_included.chain_id,
                target_included.orderbook_address,
            ),
        );
        lookup.insert(
            target_skipped.clone(),
            sample_runner_target(
                "optimism",
                target_skipped.chain_id,
                target_skipped.orderbook_address,
            ),
        );

        let mut skipped_outcome = sample_outcome(&target_skipped, "skipped.sql.gz");
        skipped_outcome.exported_dump = None;
        let successes = vec![
            skipped_outcome,
            sample_outcome(&target_included, "included.sql.gz"),
        ];

        let base_url = Url::parse("https://releases.example.com").unwrap();
        let manifest =
            build_manifest(&successes, &lookup, &base_url).expect("manifest build succeeds");

        assert!(
            !manifest.networks.contains_key("optimism"),
            "orderbook without dump should be ignored"
        );
        let arbitrum = manifest
            .networks
            .get("arbitrum")
            .expect("network with dump should exist");
        assert_eq!(arbitrum.orderbooks.len(), 1);
        assert_eq!(
            arbitrum.orderbooks[0].address,
            target_included.orderbook_address
        );
    }

    #[test]
    fn build_manifest_happy_path_multiple_networks() {
        let target_a = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000Aa1"),
        };
        let target_b = OrderbookIdentifier {
            chain_id: 10,
            orderbook_address: address!("0x0000000000000000000000000000000000000Bb2"),
        };

        let mut lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        lookup.insert(
            target_a.clone(),
            sample_runner_target("anvil", target_a.chain_id, target_a.orderbook_address),
        );
        lookup.insert(
            target_b.clone(),
            sample_runner_target("optimism", target_b.chain_id, target_b.orderbook_address),
        );

        let successes = vec![
            sample_outcome(&target_a, "dump-a.sql.gz"),
            sample_outcome(&target_b, "dump-b.sql.gz"),
        ];

        let base_url = Url::parse("https://releases.example.com").unwrap();
        let manifest =
            build_manifest(&successes, &lookup, &base_url).expect("manifest build succeeds");

        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.db_schema_version, 1);
        assert_eq!(manifest.networks.len(), 2);

        let anvil = manifest.networks.get("anvil").expect("anvil network");
        assert_eq!(anvil.chain_id, 42161);
        assert_eq!(anvil.orderbooks.len(), 1);
        let expected_anvil = format!(
            "https://releases.example.com/{}-{}.sql.gz",
            target_a.chain_id, target_a.orderbook_address
        );
        assert_eq!(anvil.orderbooks[0].dump_url.as_str(), expected_anvil);

        let optimism = manifest.networks.get("optimism").expect("optimism network");
        assert_eq!(optimism.chain_id, 10);
        assert_eq!(optimism.orderbooks.len(), 1);
        let expected_optimism = format!(
            "https://releases.example.com/{}-{}.sql.gz",
            target_b.chain_id, target_b.orderbook_address
        );
        assert_eq!(optimism.orderbooks[0].dump_url.as_str(), expected_optimism);
    }

    #[test]
    fn build_manifest_errors_on_chain_id_mismatch() {
        let target_a = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000cc1"),
        };
        let target_b = OrderbookIdentifier {
            chain_id: 10,
            orderbook_address: address!("0x0000000000000000000000000000000000000cc2"),
        };

        let mut lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        lookup.insert(
            target_a.clone(),
            sample_runner_target("shared", target_a.chain_id, target_a.orderbook_address),
        );
        lookup.insert(
            target_b.clone(),
            sample_runner_target("shared", target_b.chain_id, target_b.orderbook_address),
        );

        let successes = vec![
            sample_outcome(&target_a, "dump-a.sql.gz"),
            sample_outcome(&target_b, "dump-b.sql.gz"),
        ];

        let base_url = Url::parse("https://releases.example.com").unwrap();
        let err = build_manifest(&successes, &lookup, &base_url)
            .expect_err("shared network key with different chain ids should error");

        match err {
            LocalDbError::RunnerNetworkChainIdMismatch {
                network_key,
                expected,
                found,
            } => {
                assert_eq!(network_key, "shared");
                assert_eq!(expected, target_a.chain_id);
                assert_eq!(found, target_b.chain_id);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn build_manifest_sorts_orderbooks_within_network() {
        let target_a = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000aa2"),
        };
        let target_b = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000aa1"),
        };

        let mut lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        lookup.insert(
            target_a.clone(),
            sample_runner_target("anvil", target_a.chain_id, target_a.orderbook_address),
        );
        lookup.insert(
            target_b.clone(),
            sample_runner_target("anvil", target_b.chain_id, target_b.orderbook_address),
        );

        // Intentionally provide outcomes out of order to ensure sorting occurs.
        let successes = vec![
            sample_outcome(&target_a, "dump-a.sql.gz"),
            sample_outcome(&target_b, "dump-b.sql.gz"),
        ];

        let base_url = Url::parse("https://releases.example.com").unwrap();
        let manifest =
            build_manifest(&successes, &lookup, &base_url).expect("manifest build succeeds");
        let anvil = manifest.networks.get("anvil").expect("anvil network");
        let addresses: Vec<_> = anvil.orderbooks.iter().map(|ob| ob.address).collect();
        assert_eq!(
            addresses,
            vec![
                address!("0x0000000000000000000000000000000000000aa1"),
                address!("0x0000000000000000000000000000000000000aa2")
            ]
        );
    }

    #[test]
    fn build_manifest_errors_on_missing_target() {
        let ob_id = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000aa1"),
        };
        let lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        let successes = vec![sample_outcome(&ob_id, "dump.sql.gz")];

        let base_url = Url::parse("https://releases.example.com").unwrap();

        let err = build_manifest(&successes, &lookup, &base_url)
            .expect_err("missing target should error");
        match err {
            LocalDbError::MissingRunnerTarget {
                chain_id,
                orderbook_address,
            } => {
                assert_eq!(chain_id, ob_id.chain_id);
                assert_eq!(orderbook_address, ob_id.orderbook_address);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn build_manifest_errors_on_invalid_end_block_hash() {
        let ob_id = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000dd1"),
        };

        let mut lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        lookup.insert(
            ob_id.clone(),
            sample_runner_target("arbitrum", ob_id.chain_id, ob_id.orderbook_address),
        );

        let mut invalid_outcome = sample_outcome(&ob_id, "dump.sql.gz");
        invalid_outcome
            .exported_dump
            .as_mut()
            .expect("export present")
            .end_block_hash = "not-a-hex-string".to_string();

        let base_url = Url::parse("https://releases.example.com").unwrap();
        let err = build_manifest(&[invalid_outcome], &lookup, &base_url)
            .expect_err("invalid hash should error");

        match err {
            LocalDbError::FromHexError(_) => {}
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn build_manifest_returns_empty_manifest_for_no_successes() {
        let base_url = Url::parse("https://releases.example.com").unwrap();
        let manifest =
            build_manifest(&[], &HashMap::new(), &base_url).expect("empty input succeeds");

        assert!(manifest.networks.is_empty());
        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.db_schema_version, 1);
    }

    #[test]
    fn build_manifest_orders_networks_alphabetically() {
        let target_devnet = OrderbookIdentifier {
            chain_id: 1,
            orderbook_address: address!("0x0000000000000000000000000000000000000aa1"),
        };
        let target_testnet = OrderbookIdentifier {
            chain_id: 10,
            orderbook_address: address!("0x0000000000000000000000000000000000000bb1"),
        };

        let mut lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        lookup.insert(
            target_devnet.clone(),
            sample_runner_target(
                "devnet",
                target_devnet.chain_id,
                target_devnet.orderbook_address,
            ),
        );
        lookup.insert(
            target_testnet.clone(),
            sample_runner_target(
                "testnet",
                target_testnet.chain_id,
                target_testnet.orderbook_address,
            ),
        );

        let successes = vec![
            sample_outcome(&target_testnet, "testnet-dump.sql.gz"),
            sample_outcome(&target_devnet, "devnet-dump.sql.gz"),
        ];

        let base_url = Url::parse("https://releases.example.com/").unwrap();
        let manifest =
            build_manifest(&successes, &lookup, &base_url).expect("manifest build succeeds");

        let yaml = manifest
            .to_yaml_string()
            .expect("yaml serialization succeeds");
        let devnet_pos = yaml.find("devnet").expect("devnet key present");
        let testnet_pos = yaml.find("testnet").expect("testnet key present");
        assert!(
            devnet_pos < testnet_pos,
            "expected alphabetical ordering, got yaml: {yaml}"
        );
    }

    #[test]
    fn build_dump_url_trims_trailing_slash() {
        let base_url = Url::parse("https://releases.example.com/releases/").unwrap();
        let address = address!("0x0000000000000000000000000000000000000fff");

        let url =
            build_dump_url(&base_url, 42161, address).expect("url construction should succeed");
        let expected = format!(
            "https://releases.example.com/releases/42161-{}.sql.gz",
            address
        );
        assert_eq!(url.as_str(), expected);
    }

    #[tokio::test]
    async fn write_manifest_to_path_writes_yaml() {
        let ob_id = OrderbookIdentifier {
            chain_id: 42161,
            orderbook_address: address!("0x0000000000000000000000000000000000000aa1"),
        };
        let mut lookup: HashMap<OrderbookIdentifier, RunnerTarget> = HashMap::new();
        lookup.insert(
            ob_id.clone(),
            sample_runner_target("anvil", ob_id.chain_id, ob_id.orderbook_address),
        );
        let successes = vec![sample_outcome(&ob_id, "dump.sql.gz")];

        let base_url = Url::parse("https://releases.example.com").unwrap();
        let manifest =
            build_manifest(&successes, &lookup, &base_url).expect("manifest build succeeds");

        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("nested/manifest.yaml");
        write_manifest_to_path(&manifest, &manifest_path)
            .await
            .expect("write succeeds");

        let contents = std::fs::read_to_string(&manifest_path).expect("read manifest");
        assert!(
            contents.contains("manifest-version"),
            "expected manifest header"
        );
        assert!(contents.contains("anvil"), "expected network key");
    }
}
