use super::{bootstrap::ClientBootstrapAdapter, status::ClientStatusBus};
use crate::local_db::{
    pipeline::{
        adapters::{
            apply::DefaultApplyPipeline, events::DefaultEventsPipeline,
            tokens::DefaultTokensPipeline, window::DefaultWindowPipeline,
        },
        engine::SyncEngine,
        runner::{
            build_runner_targets, download_and_gunzip, get_manifests, lookup_manifest_entry,
            parse_runner_settings, ParsedRunnerSettings, RunnerTarget,
        },
        BootstrapPipeline, SyncOutcome,
    },
    query::LocalDbQueryExecutor,
    LocalDbError,
};
use futures::future::try_join_all;
use rain_orderbook_app_settings::{
    local_db_manifest::DB_SCHEMA_VERSION, remote::manifest::ManifestMap,
};

#[derive(Debug, Clone)]
pub struct ClientRunner {
    settings: ParsedRunnerSettings,
    base_targets: Vec<RunnerTarget>,
    manifest_map: ManifestMap,
    manifests_loaded: bool,
    has_bootstrapped: bool,
}

impl ClientRunner {
    pub fn new(settings_yaml: String) -> Result<Self, LocalDbError> {
        let settings = parse_runner_settings(&settings_yaml)?;
        let base_targets = build_runner_targets(&settings.orderbooks, &settings.syncs)?;

        Ok(Self {
            settings,
            base_targets,
            manifest_map: ManifestMap::new(),
            manifests_loaded: false,
            has_bootstrapped: false,
        })
    }

    pub async fn run<DB>(&mut self, db: &DB) -> Result<Vec<SyncOutcome>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized + Sync,
    {
        if !self.manifests_loaded {
            self.manifest_map = get_manifests(&self.settings.orderbooks).await?;
            self.manifests_loaded = true;
        }

        let mut targets = self.base_targets.clone();

        if !self.has_bootstrapped {
            let bootstrap = ClientBootstrapAdapter::new();
            bootstrap.runner_run(db, Some(DB_SCHEMA_VERSION)).await?;
            targets = self.provision_dumps(targets).await?;
        }

        let outcomes = self.execute_targets(db, targets).await?;
        self.has_bootstrapped = true;
        Ok(outcomes)
    }

    async fn provision_dumps(
        &self,
        targets: Vec<RunnerTarget>,
    ) -> Result<Vec<RunnerTarget>, LocalDbError> {
        let manifest_map = &self.manifest_map;
        let futures = targets.into_iter().map(|mut target| async move {
            if let Some(entry) = lookup_manifest_entry(manifest_map, &target) {
                let dump_sql = download_and_gunzip(&entry.dump_url).await?;
                target.inputs.dump_str = Some(dump_sql);
            }
            Ok::<RunnerTarget, LocalDbError>(target)
        });

        try_join_all(futures).await
    }

    async fn execute_targets<DB>(
        &self,
        db: &DB,
        targets: Vec<RunnerTarget>,
    ) -> Result<Vec<SyncOutcome>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized + Sync,
    {
        if targets.is_empty() {
            return Ok(vec![]);
        }

        let futures = targets.into_iter().map(|target| async move {
            let events =
                DefaultEventsPipeline::with_regular_rpcs(target.inputs.metadata_rpcs.clone())?;
            let engine = SyncEngine::new(
                ClientBootstrapAdapter::new(),
                DefaultWindowPipeline::new(),
                events,
                DefaultTokensPipeline::new(),
                DefaultApplyPipeline::new(),
                ClientStatusBus::new(),
            );
            engine.run(db, &target.inputs).await
        });

        try_join_all(futures).await
    }
}
