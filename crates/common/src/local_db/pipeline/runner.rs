use flate2::read::GzDecoder;
use rain_orderbook_app_settings::yaml::YamlParsable;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use url::Url;

use crate::local_db::fetch::FetchConfig;
use crate::local_db::pipeline::engine::SyncInputs;
use crate::local_db::pipeline::{FinalityConfig, SyncConfig, TargetKey, WindowOverrides};
use crate::local_db::LocalDbError;
use rain_orderbook_app_settings::local_db_manifest::ManifestOrderbook;
use rain_orderbook_app_settings::local_db_sync::LocalDbSyncCfg;
use rain_orderbook_app_settings::orderbook::OrderbookCfg;
use rain_orderbook_app_settings::remote::manifest::{fetch_multiple_manifests, ManifestMap};
use rain_orderbook_app_settings::yaml::orderbook::{OrderbookYaml, OrderbookYamlValidation};

/// Parsed settings required by the runner orchestration layer.
#[derive(Debug, Clone)]
pub struct ParsedRunnerSettings {
    pub orderbooks: HashMap<String, OrderbookCfg>,
    pub syncs: HashMap<String, LocalDbSyncCfg>,
}

/// Scheduling metadata for a single engine invocation.
#[derive(Debug, Clone)]
pub struct RunnerTarget {
    pub orderbook_key: String,
    pub manifest_url: Url,
    pub network_key: String,
    pub inputs: SyncInputs,
}

/// Parses the provided YAML string into orderbooks and per-network sync settings.
pub fn parse_runner_settings(settings_yaml: &str) -> Result<ParsedRunnerSettings, LocalDbError> {
    let orderbook_yaml = OrderbookYaml::new(
        vec![settings_yaml.to_owned()],
        OrderbookYamlValidation::default(),
    )?;
    let orderbooks = orderbook_yaml.get_orderbooks()?;
    let syncs = orderbook_yaml.get_local_db_syncs()?;

    Ok(ParsedRunnerSettings { orderbooks, syncs })
}

fn map_sync_to_engine(
    sync: &LocalDbSyncCfg,
) -> Result<(FetchConfig, FinalityConfig), LocalDbError> {
    let fetch = FetchConfig::new(
        sync.batch_size as u64,
        sync.max_concurrent_batches as usize,
        sync.max_concurrent_batches as usize,
        sync.retry_attempts as usize,
    )?;
    let finality = FinalityConfig {
        depth: sync.finality_depth,
    };
    Ok((fetch, finality))
}

/// Builds runner targets for all configured orderbooks.
pub fn build_runner_targets(
    orderbooks: &HashMap<String, OrderbookCfg>,
    syncs: &HashMap<String, LocalDbSyncCfg>,
) -> Result<Vec<RunnerTarget>, LocalDbError> {
    let mut targets = Vec::with_capacity(orderbooks.len());
    for (key, orderbook) in orderbooks {
        let network_key = orderbook.network.key.clone();
        let sync_cfg =
            syncs
                .get(&network_key)
                .ok_or_else(|| LocalDbError::MissingLocalDbSyncForNetwork {
                    network: network_key.clone(),
                })?;
        let (fetch, finality) = map_sync_to_engine(sync_cfg)?;
        let inputs = SyncInputs {
            target: TargetKey {
                chain_id: orderbook.network.chain_id,
                orderbook_address: orderbook.address,
            },
            metadata_rpcs: orderbook.network.rpcs.clone(),
            cfg: SyncConfig {
                deployment_block: orderbook.deployment_block,
                fetch,
                finality,
                window_overrides: WindowOverrides::default(),
            },
            dump_str: None,
        };

        targets.push(RunnerTarget {
            orderbook_key: key.clone(),
            manifest_url: orderbook.local_db_remote.url.clone(),
            network_key,
            inputs,
        });
    }

    Ok(targets)
}

/// Convenience helper retained for existing tests/utilities.
pub fn build_sync_inputs_from_yaml(settings_yaml: &str) -> Result<Vec<SyncInputs>, LocalDbError> {
    let parsed = parse_runner_settings(settings_yaml)?;
    let targets = build_runner_targets(&parsed.orderbooks, &parsed.syncs)?;
    Ok(targets.into_iter().map(|target| target.inputs).collect())
}

/// Groups runner targets by network key for per-network scheduling.
pub fn group_targets_by_network(targets: &[RunnerTarget]) -> HashMap<String, Vec<RunnerTarget>> {
    let mut grouped: HashMap<String, Vec<RunnerTarget>> = HashMap::new();
    for target in targets.iter().cloned() {
        grouped
            .entry(target.network_key.clone())
            .or_default()
            .push(target);
    }
    grouped
}

fn collect_manifest_urls(orderbooks: &HashMap<String, OrderbookCfg>) -> Vec<Url> {
    let mut urls = Vec::new();
    let mut seen = HashSet::new();
    for orderbook in orderbooks.values() {
        let url = orderbook.local_db_remote.url.clone();
        if seen.insert(url.clone()) {
            urls.push(url);
        }
    }
    urls
}

/// Fetches manifests for all distinct remotes referenced by the orderbooks.
pub async fn get_manifests(
    orderbooks: &HashMap<String, OrderbookCfg>,
) -> Result<ManifestMap, LocalDbError> {
    let urls = collect_manifest_urls(orderbooks);
    if urls.is_empty() {
        return Ok(HashMap::new());
    }

    fetch_multiple_manifests(urls)
        .await
        .map_err(LocalDbError::ManifestFetch)
}

/// Downloads a gzip-compressed SQL dump and returns the decompressed contents.
pub async fn download_and_gunzip(url: &Url) -> Result<String, LocalDbError> {
    let response = reqwest::get(url.clone()).await?;
    if !response.status().is_success() {
        return Err(LocalDbError::HttpStatus {
            status: response.status().as_u16(),
        });
    }

    let bytes = response.bytes().await?.to_vec();
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}

/// Finds the manifest entry corresponding to the runner target, if present.
pub fn lookup_manifest_entry(
    manifest_map: &ManifestMap,
    target: &RunnerTarget,
) -> Option<ManifestOrderbook> {
    manifest_map
        .get(&target.manifest_url)
        .and_then(|manifest| {
            manifest.find(
                target.inputs.target.chain_id,
                target.inputs.target.orderbook_address,
            )
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, Bytes};
    use flate2::write::GzEncoder;
    use httpmock::prelude::*;
    use rain_orderbook_app_settings::local_db_manifest::{
        LocalDbManifest, ManifestNetwork, ManifestOrderbook, MANIFEST_VERSION,
    };
    use rain_orderbook_app_settings::local_db_remotes::LocalDbRemoteCfg;
    use rain_orderbook_app_settings::remote::manifest::FetchManifestError;
    use rain_orderbook_app_settings::yaml::default_document;
    use std::collections::{HashMap, HashSet};
    use std::io::Write;
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use url::Url;

    use crate::local_db::fetch::FetchConfigError;

    fn sample_settings_yaml() -> String {
        r#"
networks:
  network-a:
    rpcs:
      - https://rpc.network/a
    chain-id: 1
  network-b:
    rpcs:
      - https://rpc.network/b
    chain-id: 2
subgraphs:
  network-a: https://subgraph.network/a
  network-b: https://subgraph.network/b
local-db-remotes:
  remote-a: https://remotes.example.com/a.yaml
  remote-b: https://remotes.example.com/b.yaml
local-db-sync:
  network-a:
    batch-size: 10
    max-concurrent-batches: 5
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 50
    finality-depth: 12
  network-b:
    batch-size: 20
    max-concurrent-batches: 2
    retry-attempts: 4
    retry-delay-ms: 200
    rate-limit-delay-ms: 100
    finality-depth: 24
orderbooks:
  ob-a:
    address: 0x00000000000000000000000000000000000000a1
    network: network-a
    subgraph: network-a
    local-db-remote: remote-a
    deployment-block: 111
  ob-b:
    address: 0x00000000000000000000000000000000000000b2
    network: network-b
    subgraph: network-b
    local-db-remote: remote-b
    deployment-block: 222
  ob-c:
    address: 0x00000000000000000000000000000000000000c3
    network: network-a
    subgraph: network-a
    local-db-remote: remote-a
    deployment-block: 333
"#
        .to_string()
    }

    fn minimal_invalid_yaml() -> &'static str {
        "networks: {"
    }

    fn missing_sync_yaml() -> String {
        r#"
networks:
  mainnet:
    rpcs:
      - https://rpc.network/mainnet
    chain-id: 1
subgraphs:
  mainnet: https://subgraph.network/mainnet
local-db-remotes:
  remote: https://remotes.example.com/mainnet.yaml
orderbooks:
  book:
    address: 0x0000000000000000000000000000000000000001
    network: mainnet
    subgraph: mainnet
    local-db-remote: remote
    deployment-block: 100
"#
        .to_string()
    }

    fn parsed_settings() -> ParsedRunnerSettings {
        parse_runner_settings(&sample_settings_yaml()).expect("valid sample YAML")
    }

    fn build_runtime() -> Runtime {
        Runtime::new().expect("tokio runtime")
    }

    #[test]
    fn parse_runner_settings_happy_path() {
        let parsed = parse_runner_settings(&sample_settings_yaml()).expect("parse succeeds");
        assert_eq!(parsed.orderbooks.len(), 3);
        assert_eq!(parsed.syncs.len(), 2);
        assert!(parsed.orderbooks.contains_key("ob-a"));
        assert!(parsed.syncs.contains_key("network-a"));
    }

    #[test]
    fn parse_runner_settings_missing_sync_section() {
        let err = parse_runner_settings(&missing_sync_yaml()).unwrap_err();
        match err {
            LocalDbError::SettingsYaml(_) => {}
            other => panic!("expected SettingsYaml error, got {other:?}"),
        }
    }

    #[test]
    fn parse_runner_settings_invalid_yaml() {
        let err = parse_runner_settings(minimal_invalid_yaml()).unwrap_err();
        let is_yaml_scan = matches!(err, LocalDbError::YamlScan(_));
        let is_missing_networks = matches!(
            err,
            LocalDbError::SettingsYaml(ref yaml_err)
                if matches!(yaml_err, rain_orderbook_app_settings::yaml::YamlError::Field { .. })
        );
        assert!(
            is_yaml_scan || is_missing_networks,
            "expected YAML parsing error, got {err:?}"
        );
    }

    #[test]
    fn map_sync_to_engine_success() {
        let sync = LocalDbSyncCfg {
            document: default_document(),
            key: "network-a".to_string(),
            batch_size: 25,
            max_concurrent_batches: 4,
            retry_attempts: 6,
            retry_delay_ms: 0,
            rate_limit_delay_ms: 0,
            finality_depth: 32,
        };

        let (fetch, finality) = map_sync_to_engine(&sync).expect("map succeeds");
        assert_eq!(fetch.chunk_size(), 25);
        assert_eq!(fetch.max_concurrent_requests(), 4);
        assert_eq!(fetch.max_concurrent_blocks(), 4);
        assert_eq!(fetch.max_retry_attempts(), 6);
        assert_eq!(finality.depth, 32);
    }

    fn assert_fetch_config_error(sync: LocalDbSyncCfg, expected: FetchConfigError) {
        let err = map_sync_to_engine(&sync).unwrap_err();
        match err {
            LocalDbError::FetchConfigError(actual) => assert_eq!(actual, expected),
            other => panic!("expected fetch config error, got {other:?}"),
        }
    }

    #[test]
    fn map_sync_to_engine_zero_batch_size_is_error() {
        let sync = LocalDbSyncCfg {
            batch_size: 0,
            ..LocalDbSyncCfg::default()
        };
        assert_fetch_config_error(sync, FetchConfigError::ChunkSizeZero(0));
    }

    #[test]
    fn map_sync_to_engine_zero_concurrency_is_error() {
        let sync = LocalDbSyncCfg {
            max_concurrent_batches: 0,
            ..LocalDbSyncCfg::default()
        };
        assert_fetch_config_error(sync, FetchConfigError::MaxConcurrentRequestsZero(0usize));
    }

    #[test]
    fn map_sync_to_engine_zero_retry_attempts_is_error() {
        let sync = LocalDbSyncCfg {
            retry_attempts: 0,
            ..LocalDbSyncCfg::default()
        };
        assert_fetch_config_error(sync, FetchConfigError::MaxRetryAttemptsZero(0));
    }

    #[test]
    fn build_runner_targets_success() {
        let parsed = parsed_settings();
        let targets =
            build_runner_targets(&parsed.orderbooks, &parsed.syncs).expect("targets build");

        assert_eq!(targets.len(), 3);

        let target_a = targets
            .iter()
            .find(|t| t.orderbook_key == "ob-a")
            .expect("target for ob-a exists");
        assert_eq!(target_a.network_key, "network-a");
        assert_eq!(target_a.inputs.target.chain_id, 1);
        assert_eq!(
            target_a.inputs.target.orderbook_address,
            address!("00000000000000000000000000000000000000a1")
        );
        assert_eq!(target_a.inputs.cfg.deployment_block, 111);
        assert_eq!(target_a.inputs.cfg.fetch.chunk_size(), 10);
        assert_eq!(target_a.inputs.cfg.finality.depth, 12);
        assert!(target_a.inputs.cfg.window_overrides.start_block.is_none());
        assert_eq!(
            target_a.inputs.metadata_rpcs,
            vec![Url::parse("https://rpc.network/a").expect("valid rpc url")]
        );
        assert_eq!(
            target_a.manifest_url,
            Url::parse("https://remotes.example.com/a.yaml").expect("valid manifest url")
        );
    }

    #[test]
    fn build_runner_targets_missing_sync_config_is_error() {
        let parsed = parsed_settings();
        let mut syncs = parsed.syncs.clone();
        syncs.remove("network-a");

        let err = build_runner_targets(&parsed.orderbooks, &syncs).unwrap_err();
        match err {
            LocalDbError::MissingLocalDbSyncForNetwork { network } => {
                assert_eq!(network, "network-a")
            }
            other => panic!("expected MissingLocalDbSyncForNetwork, got {other:?}"),
        }
    }

    #[test]
    fn build_runner_targets_invalid_sync_config_propagates_error() {
        let parsed = parsed_settings();
        let mut syncs = parsed.syncs.clone();
        if let Some(sync) = syncs.get_mut("network-a") {
            sync.batch_size = 0;
        }

        let err = build_runner_targets(&parsed.orderbooks, &syncs).unwrap_err();
        match err {
            LocalDbError::FetchConfigError(FetchConfigError::ChunkSizeZero(0)) => {}
            other => panic!("expected fetch config error, got {other:?}"),
        }
    }

    #[test]
    fn build_sync_inputs_from_yaml_success() {
        let inputs =
            build_sync_inputs_from_yaml(&sample_settings_yaml()).expect("inputs build succeeds");
        assert_eq!(inputs.len(), 3);

        let input_b = inputs
            .iter()
            .find(|input| {
                input.target.orderbook_address
                    == address!("00000000000000000000000000000000000000b2")
            })
            .expect("input for ob-b exists");
        assert_eq!(input_b.target.chain_id, 2);
        assert_eq!(input_b.cfg.fetch.max_concurrent_requests(), 2);
    }

    #[test]
    fn build_sync_inputs_from_yaml_propagates_parse_error() {
        let err = build_sync_inputs_from_yaml(minimal_invalid_yaml()).unwrap_err();
        let is_yaml_scan = matches!(err, LocalDbError::YamlScan(_));
        let is_settings_yaml = matches!(err, LocalDbError::SettingsYaml(_));
        assert!(
            is_yaml_scan || is_settings_yaml,
            "expected YAML parsing error, got {err:?}"
        );
    }

    #[test]
    fn group_targets_by_network_groups_correctly() {
        let parsed = parsed_settings();
        let targets = build_runner_targets(&parsed.orderbooks, &parsed.syncs).unwrap();
        let grouped = group_targets_by_network(&targets);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("network-a").map(|v| v.len()), Some(2));
        assert_eq!(grouped.get("network-b").map(|v| v.len()), Some(1));
    }

    #[test]
    fn group_targets_by_network_empty_input() {
        let grouped = group_targets_by_network(&[]);
        assert!(grouped.is_empty());
    }

    fn collect_urls_from_orderbooks(orderbooks: &HashMap<String, OrderbookCfg>) -> Vec<Url> {
        collect_manifest_urls(orderbooks)
    }

    #[test]
    fn collect_manifest_urls_deduplicates() {
        let parsed = parsed_settings();
        let urls = collect_urls_from_orderbooks(&parsed.orderbooks);

        assert_eq!(urls.len(), 2);
        let unique: HashSet<Url> = urls.into_iter().collect();
        assert!(
            unique.contains(&Url::parse("https://remotes.example.com/a.yaml").expect("valid url"))
        );
        assert!(
            unique.contains(&Url::parse("https://remotes.example.com/b.yaml").expect("valid url"))
        );
    }

    #[test]
    fn collect_manifest_urls_handles_empty_orderbooks() {
        let urls = collect_urls_from_orderbooks(&HashMap::new());
        assert!(urls.is_empty());
    }

    fn update_remote_url(orderbooks: &mut HashMap<String, OrderbookCfg>, key: &str, url: &Url) {
        if let Some(orderbook) = orderbooks.get_mut(key) {
            let remote = LocalDbRemoteCfg {
                document: default_document(),
                key: orderbook.local_db_remote.key.clone(),
                url: url.clone(),
            };
            orderbook.local_db_remote = Arc::new(remote);
        }
    }

    #[test]
    fn get_manifests_returns_empty_map_for_no_orderbooks() {
        let rt = build_runtime();
        let manifests = rt
            .block_on(get_manifests(&HashMap::new()))
            .expect("empty map");
        assert!(manifests.is_empty());
    }

    #[test]
    fn get_manifests_fetches_unique_urls() {
        let rt = build_runtime();

        let server_one = MockServer::start();
        let server_two = MockServer::start();

        let manifest_one = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "https://example.com/dump1.sql.gz"
        end-block: 100
        end-block-hash: "0x01"
        end-block-time-ms: 1000
"#;

        let manifest_two = r#"
manifest-version: 1
db-schema-version: 1
networks:
  goerli:
    chain-id: 5
    orderbooks:
      - address: "0x0000000000000000000000000000000000000002"
        dump-url: "https://example.com/dump2.sql.gz"
        end-block: 200
        end-block-hash: "0x02"
        end-block-time-ms: 2000
"#;

        server_one.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(manifest_one);
        });

        server_two.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(manifest_two);
        });

        let mut parsed = parsed_settings();
        let url_one = Url::parse(&server_one.base_url()).unwrap();
        let url_two = Url::parse(&server_two.base_url()).unwrap();
        update_remote_url(&mut parsed.orderbooks, "ob-a", &url_one);
        update_remote_url(&mut parsed.orderbooks, "ob-c", &url_one);
        update_remote_url(&mut parsed.orderbooks, "ob-b", &url_two);

        // Ensure both remotes are referenced
        let manifests = rt
            .block_on(get_manifests(&parsed.orderbooks))
            .expect("manifests fetched");
        assert_eq!(manifests.len(), 2);
        assert!(manifests.contains_key(&url_one));
        assert!(manifests.contains_key(&url_two));
    }

    #[test]
    fn get_manifests_propagates_manifest_error() {
        let rt = build_runtime();
        let server_one = MockServer::start();
        let server_two = MockServer::start();

        let manifest_one = format!(
            r#"
manifest-version: {MANIFEST_VERSION}
db-schema-version: 1
networks: {{}}
"#
        );
        let manifest_two = r#"
manifest-version: 2
db-schema-version: 1
networks: {}
"#;

        server_one.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(&manifest_one);
        });

        server_two.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(manifest_two);
        });

        let mut parsed = parsed_settings();
        let url_one = Url::parse(&server_one.base_url()).unwrap();
        let url_two = Url::parse(&server_two.base_url()).unwrap();
        update_remote_url(&mut parsed.orderbooks, "ob-a", &url_one);
        update_remote_url(&mut parsed.orderbooks, "ob-c", &url_one);
        update_remote_url(&mut parsed.orderbooks, "ob-b", &url_two);

        let err = rt.block_on(get_manifests(&parsed.orderbooks)).unwrap_err();
        match err {
            LocalDbError::ManifestFetch(FetchManifestError::Yaml(_)) => {}
            other => panic!("expected YAML manifest error, got {other:?}"),
        }
    }

    #[test]
    fn get_manifests_propagates_network_error() {
        let rt = build_runtime();
        let mut parsed = parsed_settings();
        let unreachable = Url::parse("nosuch://unreachable.example/manifest.yaml").unwrap();
        update_remote_url(&mut parsed.orderbooks, "ob-a", &unreachable);
        update_remote_url(&mut parsed.orderbooks, "ob-b", &unreachable);
        update_remote_url(&mut parsed.orderbooks, "ob-c", &unreachable);

        let err = rt.block_on(get_manifests(&parsed.orderbooks)).unwrap_err();
        match err {
            LocalDbError::ManifestFetch(FetchManifestError::ReqwestError(_)) => {}
            other => panic!("expected manifest fetch reqwest error, got {other:?}"),
        }
    }

    #[test]
    fn download_and_gunzip_success() {
        let rt = build_runtime();
        let server = MockServer::start();
        let body_text = "CREATE TABLE orders (...);";

        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(body_text.as_bytes())
            .expect("encode gzip");
        let gzipped = encoder.finish().expect("finish gzip");

        server.mock(|when, then| {
            when.method(GET).path("/dump.sql.gz");
            then.status(200)
                .header("content-type", "application/gzip")
                .body(gzipped.clone());
        });

        let url = Url::parse(&format!("{}/dump.sql.gz", server.base_url())).unwrap();
        let contents = rt
            .block_on(download_and_gunzip(&url))
            .expect("decompressed");
        assert_eq!(contents, body_text);
    }

    #[test]
    fn download_and_gunzip_non_success_status() {
        let rt = build_runtime();
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/dump.sql.gz");
            then.status(404);
        });
        let url = Url::parse(&format!("{}/dump.sql.gz", server.base_url())).unwrap();
        let err = rt.block_on(download_and_gunzip(&url)).unwrap_err();
        match err {
            LocalDbError::HttpStatus { status } => assert_eq!(status, 404),
            other => panic!("expected HttpStatus error, got {other:?}"),
        }
    }

    #[test]
    fn download_and_gunzip_invalid_gzip() {
        let rt = build_runtime();
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/dump.sql.gz");
            then.status(200).body("not-gzip");
        });
        let url = Url::parse(&format!("{}/dump.sql.gz", server.base_url())).unwrap();
        let err = rt.block_on(download_and_gunzip(&url)).unwrap_err();
        match err {
            LocalDbError::IoError(_) => {}
            other => panic!("expected IoError due to decompression failure, got {other:?}"),
        }
    }

    #[test]
    fn download_and_gunzip_network_error() {
        let rt = build_runtime();
        let url = Url::parse("nosuch://unreachable.example/dump.sql.gz").unwrap();

        let err = rt.block_on(download_and_gunzip(&url)).unwrap_err();
        match err {
            LocalDbError::Http(_) => {}
            other => panic!("expected Http error for unreachable URL, got {other:?}"),
        }
    }

    fn sample_runner_target() -> (RunnerTarget, super::ManifestMap) {
        let parsed = parsed_settings();
        let targets = build_runner_targets(&parsed.orderbooks, &parsed.syncs).unwrap();
        let target = targets
            .into_iter()
            .find(|t| t.orderbook_key == "ob-a")
            .expect("target ob-a");

        let manifest_entry = ManifestOrderbook {
            address: target.inputs.target.orderbook_address,
            dump_url: Url::parse("https://example.com/dump.sql.gz").unwrap(),
            end_block: 123,
            end_block_hash: Bytes::from_static(&[0x01, 0x02, 0x03]),
            end_block_time_ms: 456,
        };

        let manifest = LocalDbManifest {
            manifest_version: MANIFEST_VERSION,
            db_schema_version: 1,
            networks: HashMap::from([(
                "network-a".to_string(),
                ManifestNetwork {
                    chain_id: target.inputs.target.chain_id,
                    orderbooks: vec![manifest_entry.clone()],
                },
            )]),
        };

        let map = HashMap::from([(target.manifest_url.clone(), manifest)]);
        (target, map)
    }

    #[test]
    fn lookup_manifest_entry_found() {
        let (target, manifest_map) = sample_runner_target();
        let entry = lookup_manifest_entry(&manifest_map, &target).expect("entry found");
        assert_eq!(entry.address, target.inputs.target.orderbook_address);
    }

    #[test]
    fn lookup_manifest_entry_missing_address() {
        let (mut target, mut manifest_map) = sample_runner_target();
        // Change target address to one that does not exist
        target.inputs.target.orderbook_address =
            address!("000000000000000000000000000000000000dead");
        assert!(lookup_manifest_entry(&manifest_map, &target).is_none());

        // Remove manifest entirely
        manifest_map.clear();
        assert!(lookup_manifest_entry(&manifest_map, &target).is_none());
    }

    #[test]
    fn lookup_manifest_entry_chain_id_mismatch() {
        let (target, mut manifest_map) = sample_runner_target();
        if let Some(manifest) = manifest_map.get_mut(&target.manifest_url) {
            if let Some(network) = manifest.networks.get_mut("network-a") {
                network.chain_id = target.inputs.target.chain_id + 1;
            }
        }

        assert!(lookup_manifest_entry(&manifest_map, &target).is_none());
    }
}
