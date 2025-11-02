use alloy::primitives::Address;
use rain_orderbook_app_settings::local_db_manifest::ManifestOrderbook;
use rain_orderbook_common::local_db::pipeline::{
    adapters::{
        apply::DefaultApplyPipeline, events::DefaultEventsPipeline, tokens::DefaultTokensPipeline,
        window::DefaultWindowPipeline,
    },
    engine::SyncInputs,
    runner::{
        build_runner_targets, lookup_manifest_entry, parse_runner_settings, ParsedRunnerSettings,
        RunnerEnvironment, RunnerTarget,
    },
    ApplyPipeline, BootstrapPipeline, EventsPipeline, StatusBus, SyncOutcome, TokensPipeline,
    WindowPipeline,
};
use rain_orderbook_common::local_db::LocalDbError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task::{JoinSet, LocalSet};
use tracing::error;

use crate::commands::local_db::executor::RusqliteExecutor;
use crate::commands::local_db::pipeline::{
    bootstrap::ProducerBootstrapAdapter, status::ProducerStatusBus,
};

use super::wiring::default_environment;

pub struct ProducerRunner<B, W, E, T, A, S> {
    settings: ParsedRunnerSettings,
    targets: Vec<RunnerTarget>,
    out_root: PathBuf,
    environment: RunnerEnvironment<B, W, E, T, A, S>,
}

impl<B, W, E, T, A, S> ProducerRunner<B, W, E, T, A, S>
where
    B: BootstrapPipeline + 'static,
    W: WindowPipeline + 'static,
    E: EventsPipeline + 'static,
    T: TokensPipeline + 'static,
    A: ApplyPipeline + 'static,
    S: StatusBus + 'static,
{
    pub fn with_environment(
        settings_yaml: String,
        out_root: PathBuf,
        environment: RunnerEnvironment<B, W, E, T, A, S>,
    ) -> Result<Self, LocalDbError> {
        let settings = parse_runner_settings(&settings_yaml)?;
        let targets = build_runner_targets(&settings.orderbooks, &settings.syncs)?;

        Ok(Self {
            settings,
            targets,
            out_root,
            environment,
        })
    }

    pub async fn run(&self) -> Result<ProducerRunReport, LocalDbError> {
        let manifest_map = Arc::new(
            self.environment
                .fetch_manifests(&self.settings.orderbooks)
                .await?,
        );
        let targets = self.targets.clone();
        let out_root = self.out_root.clone();
        let environment = self.environment.clone();

        let local = LocalSet::new();
        local
            .run_until(async move {
                let mut tasks = JoinSet::new();
                for target in targets {
                    let manifest_map = Arc::clone(&manifest_map);
                    let out_root = out_root.clone();
                    let environment = environment.clone();
                    tasks.spawn_local(async move {
                        let manifest_entry = lookup_manifest_entry(manifest_map.as_ref(), &target);
                        let chain_id = target.inputs.target.chain_id;
                        let orderbook_address = target.inputs.target.orderbook_address;
                        match run_orderbook_job::<B, W, E, T, A, S>(
                            target,
                            manifest_entry,
                            environment,
                            out_root,
                        )
                        .await
                        {
                            Ok(outcome) => Ok(outcome),
                            Err(error) => Err(ProducerJobFailure {
                                chain_id: Some(chain_id),
                                orderbook_address: Some(orderbook_address),
                                error,
                            }),
                        }
                    });
                }

                let mut successes = Vec::new();
                let mut failures = Vec::new();
                while let Some(result) = tasks.join_next().await {
                    match result {
                        Ok(Ok(outcome)) => successes.push(outcome),
                        Ok(Err(failure)) => {
                            error!(
                                address = ?failure.orderbook_address,
                                error = %failure.error,
                                "producer job failed (chain_id={:?})",
                                failure.chain_id,
                            );
                            failures.push(failure);
                        }
                        Err(join_err) => {
                            let error = LocalDbError::from(join_err);
                            error!(
                                error = %error,
                                "producer job panicked or was cancelled before completion"
                            );
                            failures.push(ProducerJobFailure {
                                chain_id: None,
                                orderbook_address: None,
                                error,
                            });
                        }
                    }
                }
                Ok(ProducerRunReport {
                    successes,
                    failures,
                })
            })
            .await
    }
}

impl
    ProducerRunner<
        ProducerBootstrapAdapter,
        DefaultWindowPipeline,
        DefaultEventsPipeline,
        DefaultTokensPipeline,
        DefaultApplyPipeline,
        ProducerStatusBus,
    >
{
    pub fn new(
        settings_yaml: String,
        out_root: PathBuf,
        hypersync_token: String,
    ) -> Result<Self, LocalDbError> {
        let environment = default_environment(hypersync_token);
        Self::with_environment(settings_yaml, out_root, environment)
    }
}

#[derive(Debug)]
pub struct ProducerOutcome {
    pub outcome: SyncOutcome,
    pub exported_dump: ExportMetadata,
}

#[derive(Debug)]
pub struct ProducerRunReport {
    pub successes: Vec<ProducerOutcome>,
    pub failures: Vec<ProducerJobFailure>,
}

#[derive(Debug)]
pub struct ProducerJobFailure {
    pub chain_id: Option<u32>,
    pub orderbook_address: Option<Address>,
    pub error: LocalDbError,
}

#[derive(Debug)]
pub struct ExportMetadata {
    pub dump_path: PathBuf,
    pub end_block: u64,
    pub end_block_hash: String,
    pub end_block_time_ms: u64,
}

async fn run_orderbook_job<B, W, E, T, A, S>(
    target: RunnerTarget,
    manifest_entry: Option<ManifestOrderbook>,
    environment: RunnerEnvironment<B, W, E, T, A, S>,
    out_root: PathBuf,
) -> Result<ProducerOutcome, LocalDbError>
where
    B: BootstrapPipeline + 'static,
    W: WindowPipeline + 'static,
    E: EventsPipeline + 'static,
    T: TokensPipeline + 'static,
    A: ApplyPipeline + 'static,
    S: StatusBus + 'static,
{
    let RunnerTarget {
        orderbook_key,
        manifest_url,
        network_key,
        inputs,
    } = target;

    let inputs = match manifest_entry {
        Some(entry) => {
            let dump_sql = environment.download_dump(&entry.dump_url).await?;
            SyncInputs {
                dump_str: Some(dump_sql),
                ..inputs
            }
        }
        None => inputs,
    };

    let target = RunnerTarget {
        orderbook_key,
        manifest_url,
        network_key,
        inputs,
    };

    let db_path = db_path_for_target(&out_root, &target)?;
    ensure_clean_db(&db_path)?;
    let executor = RusqliteExecutor::new(&db_path);

    let engine = environment.build_engine(&target)?.into_engine();
    let outcome = engine.run(&executor, &target.inputs).await?;
    let exported_dump = export_dump(&executor, &target, &out_root).await?;

    Ok(ProducerOutcome {
        outcome,
        exported_dump,
    })
}

fn db_path_for_target(out_root: &Path, target: &RunnerTarget) -> Result<PathBuf, LocalDbError> {
    let chain_folder = out_root.join(target.inputs.target.chain_id.to_string());
    std::fs::create_dir_all(&chain_folder)?;
    let filename = format!("{}.db", target.inputs.target.orderbook_address);
    Ok(chain_folder.join(filename))
}

fn ensure_clean_db(path: &Path) -> Result<(), LocalDbError> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

async fn export_dump(
    executor: &RusqliteExecutor,
    target: &RunnerTarget,
    out_root: &Path,
) -> Result<ExportMetadata, LocalDbError> {
    let _ = (executor, target, out_root);
    todo!("implement producer export pipeline")
}
