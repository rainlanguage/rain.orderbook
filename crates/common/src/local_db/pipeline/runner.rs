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
