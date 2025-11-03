use super::wiring::default_environment;
use crate::local_db::{
    pipeline::{
        adapters::{
            apply::DefaultApplyPipeline, events::DefaultEventsPipeline,
            tokens::DefaultTokensPipeline, window::DefaultWindowPipeline,
        },
        runner::{
            build_runner_targets, lookup_manifest_entry, parse_runner_settings,
            ParsedRunnerSettings, RunnerEnvironment, RunnerTarget,
        },
        ApplyPipeline, BootstrapPipeline, EventsPipeline, StatusBus, SyncOutcome, TokensPipeline,
        WindowPipeline,
    },
    query::LocalDbQueryExecutor,
    LocalDbError,
};
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
use crate::raindex_client::local_db::pipeline::status::ClientStatusBus;
use futures::future::try_join_all;
use rain_orderbook_app_settings::{
    local_db_manifest::DB_SCHEMA_VERSION, remote::manifest::ManifestMap,
};

pub struct ClientRunner<B, W, E, T, A, S> {
    settings: ParsedRunnerSettings,
    base_targets: Vec<RunnerTarget>,
    manifest_map: ManifestMap,
    manifests_loaded: bool,
    has_bootstrapped: bool,
    environment: RunnerEnvironment<B, W, E, T, A, S>,
}

impl<B, W, E, T, A, S> ClientRunner<B, W, E, T, A, S>
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
        environment: RunnerEnvironment<B, W, E, T, A, S>,
    ) -> Result<Self, LocalDbError> {
        let settings = parse_runner_settings(&settings_yaml)?;
        let base_targets = build_runner_targets(&settings.orderbooks, &settings.syncs)?;

        Ok(Self {
            settings,
            base_targets,
            manifest_map: ManifestMap::new(),
            manifests_loaded: false,
            has_bootstrapped: false,
            environment,
        })
    }

    pub async fn run<DB>(&mut self, db: &DB) -> Result<Vec<SyncOutcome>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized + Sync,
    {
        if !self.manifests_loaded {
            self.manifest_map = self
                .environment
                .fetch_manifests(&self.settings.orderbooks)
                .await?;
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
        let environment = self.environment.clone();
        let futures = targets.into_iter().map(move |mut target| {
            let environment = environment.clone();
            async move {
                if let Some(entry) = lookup_manifest_entry(manifest_map, &target) {
                    let dump_sql = environment.download_dump(&entry.dump_url).await?;
                    target.inputs.dump_str = Some(dump_sql);
                }
                Ok::<RunnerTarget, LocalDbError>(target)
            }
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

        let environment = self.environment.clone();
        let futures = targets.into_iter().map(move |target| {
            let environment = environment.clone();
            async move {
                let engine = environment.build_engine(&target)?.into_engine();
                engine.run(db, &target.inputs).await
            }
        });

        try_join_all(futures).await
    }
}

impl
    ClientRunner<
        ClientBootstrapAdapter,
        DefaultWindowPipeline,
        DefaultEventsPipeline,
        DefaultTokensPipeline,
        DefaultApplyPipeline,
        ClientStatusBus,
    >
{
    pub fn new(settings_yaml: String) -> Result<Self, LocalDbError> {
        let environment = default_environment();
        Self::with_environment(settings_yaml, environment)
    }
}
