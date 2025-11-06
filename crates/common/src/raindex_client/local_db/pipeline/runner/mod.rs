pub mod environment;
pub mod leadership;
pub mod scheduler;

use crate::local_db::{
    pipeline::{
        adapters::{
            apply::DefaultApplyPipeline, events::DefaultEventsPipeline,
            tokens::DefaultTokensPipeline, window::DefaultWindowPipeline,
        },
        runner::{
            environment::RunnerEnvironment,
            remotes::lookup_manifest_entry,
            utils::{
                build_runner_targets, parse_runner_settings, ParsedRunnerSettings, RunnerTarget,
            },
        },
        ApplyPipeline, BootstrapPipeline, EventsPipeline, StatusBus, SyncOutcome, TokensPipeline,
        WindowPipeline,
    },
    query::LocalDbQueryExecutor,
    LocalDbError,
};
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
use crate::raindex_client::local_db::pipeline::status::ClientStatusBus;
use environment::default_environment;
use futures::future::try_join_all;
use leadership::{DefaultLeadership, Leadership, LeadershipGuard};
use rain_orderbook_app_settings::{
    local_db_manifest::DB_SCHEMA_VERSION, remote::manifest::ManifestMap,
};

pub struct ClientRunner<B, W, E, T, A, S, L> {
    settings: ParsedRunnerSettings,
    base_targets: Vec<RunnerTarget>,
    manifest_map: ManifestMap,
    manifests_loaded: bool,
    has_bootstrapped: bool,
    environment: RunnerEnvironment<B, W, E, T, A, S>,
    leadership: L,
    leadership_guard: Option<LeadershipGuard>,
}

impl<B, W, E, T, A, S, L> ClientRunner<B, W, E, T, A, S, L>
where
    B: BootstrapPipeline + 'static,
    W: WindowPipeline + 'static,
    E: EventsPipeline + 'static,
    T: TokensPipeline + 'static,
    A: ApplyPipeline + 'static,
    S: StatusBus + 'static,
    L: Leadership + 'static,
{
    pub fn with_environment(
        settings_yaml: String,
        environment: RunnerEnvironment<B, W, E, T, A, S>,
        leadership: L,
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
            leadership,
            leadership_guard: None,
        })
    }

    pub async fn run<DB>(&mut self, db: &DB) -> Result<Vec<SyncOutcome>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized + Sync,
    {
        if self.leadership_guard.is_none() {
            match self.leadership.acquire().await? {
                Some(guard) => self.leadership_guard = Some(guard),
                None => return Ok(vec![]),
            }
        }

        let mut fetched_manifests = false;
        if !self.manifests_loaded {
            self.manifest_map = self
                .environment
                .fetch_manifests(&self.settings.orderbooks)
                .await?;
            fetched_manifests = true;
        }

        let mut targets = self.base_targets.clone();
        let needs_bootstrap = !self.has_bootstrapped;

        if needs_bootstrap {
            let bootstrap = ClientBootstrapAdapter::new();
            bootstrap.runner_run(db, Some(DB_SCHEMA_VERSION)).await?;
            targets = self.provision_dumps(targets).await?;
        }

        let outcomes = self.execute_targets(db, targets).await?;
        if fetched_manifests {
            self.manifests_loaded = true;
        }
        if needs_bootstrap {
            self.has_bootstrapped = true;
        }
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
                    target.inputs.manifest_end_block = entry.end_block;
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
        DefaultLeadership,
    >
{
    pub fn new(settings_yaml: String) -> Result<Self, LocalDbError> {
        let environment = default_environment();
        Self::with_environment(settings_yaml, environment, DefaultLeadership::new())
    }
}

#[cfg(test)]
mod tests {
    use super::leadership::LeadershipGuard;
    use super::*;
    use crate::local_db::decode::{DecodedEvent, DecodedEventData};
    use crate::local_db::fetch::FetchConfig;
    use crate::local_db::pipeline::runner::environment::{
        DumpFuture, EnginePipelines, ManifestFuture,
    };
    use crate::local_db::pipeline::{
        ApplyPipelineTargetInfo, BootstrapConfig, BootstrapPipeline, BootstrapState,
        EventsPipeline, StatusBus, SyncConfig, TargetKey, TokensPipeline, WindowPipeline,
    };
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use crate::local_db::query::fetch_db_metadata::{fetch_db_metadata_stmt, DbMetadataRow};
    use crate::local_db::query::fetch_store_addresses::fetch_store_addresses_stmt;
    use crate::local_db::query::fetch_tables::{fetch_tables_stmt, TableResponse};
    use crate::local_db::query::fetch_target_watermark::fetch_target_watermark_stmt;
    use crate::local_db::query::{FromDbJson, LocalDbQueryError, SqlStatement, SqlStatementBatch};
    use crate::local_db::LocalDbError;
    use crate::rpc_client::LogEntryResponse;
    use alloy::primitives::{address, Address, Bytes};
    use async_trait::async_trait;
    use rain_orderbook_app_settings::local_db_manifest::{
        LocalDbManifest, ManifestNetwork, ManifestOrderbook, DB_SCHEMA_VERSION, MANIFEST_VERSION,
    };
    use rain_orderbook_app_settings::orderbook::OrderbookCfg;
    use rain_orderbook_app_settings::remote::manifest::ManifestMap;
    use serde::Serialize;
    use serde_json::{json, Value};
    use std::collections::{HashMap, VecDeque};
    use std::str::FromStr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use url::Url;

    const CHAIN_ID: u32 = 42161;
    const NETWORK_KEY: &str = "anvil";
    const ORDERBOOK_KEY_A: &str = "ob-a";
    const ORDERBOOK_KEY_B: &str = "ob-b";

    const ORDERBOOK_A: Address = address!("00000000000000000000000000000000000000a1");
    const ORDERBOOK_B: Address = address!("00000000000000000000000000000000000000b2");

    #[derive(Clone, Default)]
    struct AlwaysLeadership;

    #[async_trait(?Send)]
    impl Leadership for AlwaysLeadership {
        async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
            Ok(Some(LeadershipGuard::new_noop()))
        }
    }

    #[derive(Clone, Default)]
    struct CountingLeadership {
        calls: Arc<AtomicUsize>,
    }

    impl CountingLeadership {
        fn new() -> Self {
            Self::default()
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait(?Send)]
    impl Leadership for CountingLeadership {
        async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(Some(LeadershipGuard::new_noop()))
        }
    }

    #[derive(Debug, Clone)]
    enum LeadershipAction {
        Grant,
        Skip,
        Fail(String),
    }

    #[derive(Clone)]
    struct SequenceLeadership {
        actions: Arc<Mutex<VecDeque<LeadershipAction>>>,
    }

    impl SequenceLeadership {
        fn new(actions: Vec<LeadershipAction>) -> Self {
            Self {
                actions: Arc::new(Mutex::new(VecDeque::from(actions))),
            }
        }
    }

    #[async_trait(?Send)]
    impl Leadership for SequenceLeadership {
        async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
            let action = self
                .actions
                .lock()
                .expect("actions lock poisoned")
                .pop_front()
                .unwrap_or_else(|| panic!("unexpected leadership acquire call"));
            match action {
                LeadershipAction::Grant => Ok(Some(LeadershipGuard::new_noop())),
                LeadershipAction::Skip => Ok(None),
                LeadershipAction::Fail(message) => Err(LocalDbError::CustomError(message)),
            }
        }
    }

    #[derive(Clone, Default)]
    struct Telemetry {
        manifest_fetches: Arc<AtomicUsize>,
        dump_requests: Arc<Mutex<Vec<Url>>>,
        engine_runs: Arc<Mutex<Vec<String>>>,
        bootstrap_records: Arc<Mutex<Vec<BootstrapRecord>>>,
        status_messages: Arc<Mutex<Vec<(String, String)>>>,
        persist_calls: Arc<AtomicUsize>,
        builder_inits: Arc<AtomicUsize>,
    }

    #[derive(Clone, Debug)]
    struct BootstrapRecord {
        orderbook_key: String,
        dump_sql: Option<String>,
    }

    impl Telemetry {
        fn record_manifest_fetch(&self) -> usize {
            self.manifest_fetches.fetch_add(1, Ordering::SeqCst)
        }

        fn manifest_fetch_count(&self) -> usize {
            self.manifest_fetches.load(Ordering::SeqCst)
        }

        fn record_dump(&self, url: Url) -> usize {
            let mut dumps = self.dump_requests.lock().unwrap();
            dumps.push(url);
            dumps.len()
        }

        fn dump_requests(&self) -> Vec<Url> {
            self.dump_requests.lock().unwrap().clone()
        }

        fn record_engine_run(&self, orderbook_key: &str) {
            self.engine_runs
                .lock()
                .unwrap()
                .push(orderbook_key.to_string());
        }

        fn engine_runs(&self) -> Vec<String> {
            self.engine_runs.lock().unwrap().clone()
        }

        fn record_bootstrap(&self, orderbook_key: String, dump_sql: Option<String>) {
            self.bootstrap_records
                .lock()
                .unwrap()
                .push(BootstrapRecord {
                    orderbook_key,
                    dump_sql,
                });
        }

        fn bootstrap_records(&self) -> Vec<BootstrapRecord> {
            self.bootstrap_records.lock().unwrap().clone()
        }

        fn record_status(&self, orderbook_key: &str, message: &str) {
            self.status_messages
                .lock()
                .unwrap()
                .push((orderbook_key.to_string(), message.to_string()));
        }

        fn record_persist(&self) {
            self.persist_calls.fetch_add(1, Ordering::SeqCst);
        }

        fn persist_count(&self) -> usize {
            self.persist_calls.load(Ordering::SeqCst)
        }

        fn record_builder_init(&self) {
            self.builder_inits.fetch_add(1, Ordering::SeqCst);
        }

        fn builder_inits(&self) -> usize {
            self.builder_inits.load(Ordering::SeqCst)
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum EngineBehavior {
        Success,
        ApplyFail,
    }

    #[derive(Clone, Default)]
    struct RecordingDb {
        inner: Arc<RecordingDbInner>,
    }

    #[derive(Default)]
    struct RecordingDbInner {
        json_map: Mutex<HashMap<String, JsonResponse>>,
        text_map: Mutex<HashMap<String, Result<String, LocalDbQueryError>>>,
        batch_calls: Mutex<Vec<Vec<String>>>,
        text_calls: Mutex<Vec<String>>,
    }

    #[derive(Clone)]
    enum JsonResponse {
        Value(Value),
        Error(LocalDbQueryError),
    }

    impl RecordingDb {
        fn set_json_value<T>(&self, stmt: &SqlStatement, value: T)
        where
            T: Serialize,
        {
            self.inner
                .json_map
                .lock()
                .unwrap()
                .insert(stmt.sql().to_string(), JsonResponse::Value(json!(value)));
        }

        fn set_json_raw(&self, stmt: &SqlStatement, value: Value) {
            self.inner
                .json_map
                .lock()
                .unwrap()
                .insert(stmt.sql().to_string(), JsonResponse::Value(value));
        }

        fn set_json_error(&self, stmt: &SqlStatement, err: LocalDbQueryError) {
            self.inner
                .json_map
                .lock()
                .unwrap()
                .insert(stmt.sql().to_string(), JsonResponse::Error(err));
        }

        fn batch_calls(&self) -> Vec<Vec<String>> {
            self.inner.batch_calls.lock().unwrap().clone()
        }
    }

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for RecordingDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            let statements: Vec<String> = batch
                .into_iter()
                .map(|stmt| stmt.sql().to_string())
                .collect();
            self.inner.batch_calls.lock().unwrap().push(statements);
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            let sql = stmt.sql().to_string();
            let response = self.inner.json_map.lock().unwrap().get(&sql).cloned();
            match response {
                Some(JsonResponse::Value(value)) => serde_json::from_value(value)
                    .map_err(|err| LocalDbQueryError::deserialization(err.to_string())),
                Some(JsonResponse::Error(err)) => Err(err),
                None => Err(LocalDbQueryError::database(format!(
                    "no json response configured for sql: {sql}"
                ))),
            }
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            let sql = stmt.sql().to_string();
            self.inner.text_calls.lock().unwrap().push(sql.clone());
            let response = self.inner.text_map.lock().unwrap().get(&sql).cloned();
            match response {
                Some(Ok(value)) => Ok(value),
                Some(Err(err)) => Err(err),
                None => Ok("ok".to_string()),
            }
        }
    }

    #[derive(Clone)]
    struct StubBootstrap {
        telemetry: Telemetry,
        orderbook_key: String,
    }

    impl StubBootstrap {
        fn new(telemetry: Telemetry, orderbook_key: String) -> Self {
            Self {
                telemetry,
                orderbook_key,
            }
        }
    }

    #[async_trait(?Send)]
    impl BootstrapPipeline for StubBootstrap {
        async fn ensure_schema<DB>(
            &self,
            _db: &DB,
            _db_schema_version: Option<u32>,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }

        async fn inspect_state<DB>(
            &self,
            _db: &DB,
            _target_key: &TargetKey,
        ) -> Result<BootstrapState, LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(BootstrapState {
                has_required_tables: true,
                last_synced_block: None,
            })
        }

        async fn reset_db<DB>(
            &self,
            _db: &DB,
            _db_schema_version: Option<u32>,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }

        async fn clear_orderbook_data<DB>(
            &self,
            _db: &DB,
            _target: &TargetKey,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }

        async fn engine_run<DB>(
            &self,
            _db: &DB,
            config: &BootstrapConfig,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            let dump_sql = config.dump_stmt.as_ref().map(|stmt| stmt.sql().to_string());
            self.telemetry
                .record_bootstrap(self.orderbook_key.clone(), dump_sql);
            Ok(())
        }

        async fn runner_run<DB>(
            &self,
            _db: &DB,
            _db_schema_version: Option<u32>,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct StubWindow {
        start_block: u64,
        target_block: u64,
    }

    impl StubWindow {
        fn new(start_block: u64, target_block: u64) -> Self {
            Self {
                start_block,
                target_block,
            }
        }
    }

    #[async_trait(?Send)]
    impl WindowPipeline for StubWindow {
        async fn compute<DB>(
            &self,
            _db: &DB,
            _target: &TargetKey,
            _cfg: &SyncConfig,
            _latest_block: u64,
        ) -> Result<(u64, u64), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok((self.start_block, self.target_block))
        }
    }

    #[derive(Clone)]
    struct StubEvents {
        latest_block: u64,
    }

    impl StubEvents {
        fn new(latest_block: u64) -> Self {
            Self { latest_block }
        }
    }

    #[async_trait(?Send)]
    impl EventsPipeline for StubEvents {
        async fn latest_block(&self) -> Result<u64, LocalDbError> {
            Ok(self.latest_block)
        }

        async fn block_hash(&self, _block_number: u64) -> Result<Bytes, LocalDbError> {
            Ok(Bytes::new())
        }

        async fn fetch_orderbook(
            &self,
            _orderbook_address: Address,
            _from_block: u64,
            _to_block: u64,
            _cfg: &FetchConfig,
        ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
            Ok(Vec::new())
        }

        async fn fetch_stores(
            &self,
            _store_addresses: &[Address],
            _from_block: u64,
            _to_block: u64,
            _cfg: &FetchConfig,
        ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
            Ok(Vec::new())
        }

        fn decode(
            &self,
            _logs: &[LogEntryResponse],
        ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
            Ok(Vec::new())
        }
    }

    #[derive(Clone, Copy)]
    struct StubTokens;

    #[async_trait(?Send)]
    impl TokensPipeline for StubTokens {
        async fn load_existing<DB>(
            &self,
            _db: &DB,
            _chain_id: u32,
            _orderbook_address: Address,
            _token_addrs_lower: &[Address],
        ) -> Result<
            Vec<crate::local_db::query::fetch_erc20_tokens_by_addresses::Erc20TokenRow>,
            LocalDbError,
        >
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(Vec::new())
        }

        async fn fetch_missing(
            &self,
            _missing: Vec<Address>,
            _cfg: &FetchConfig,
        ) -> Result<Vec<(Address, crate::erc20::TokenInfo)>, LocalDbError> {
            Ok(Vec::new())
        }
    }

    #[derive(Clone)]
    struct StubApply {
        telemetry: Telemetry,
        orderbook_key: String,
        fail: bool,
    }

    impl StubApply {
        fn new(telemetry: Telemetry, orderbook_key: String, fail: bool) -> Self {
            Self {
                telemetry,
                orderbook_key,
                fail,
            }
        }
    }

    #[async_trait(?Send)]
    impl ApplyPipeline for StubApply {
        fn build_batch(
            &self,
            _target_info: &ApplyPipelineTargetInfo,
            _raw_logs: &[LogEntryResponse],
            _decoded_events: &[DecodedEventData<DecodedEvent>],
            _existing_tokens: &[crate::local_db::query::fetch_erc20_tokens_by_addresses::Erc20TokenRow],
            _tokens_to_upsert: &[(Address, crate::erc20::TokenInfo)],
        ) -> Result<SqlStatementBatch, LocalDbError> {
            let mut batch = SqlStatementBatch::new();
            batch.add(SqlStatement::new("BEGIN TRANSACTION"));
            batch.add(SqlStatement::new(format!(
                "-- apply {}",
                self.orderbook_key
            )));
            batch.add(SqlStatement::new("COMMIT"));
            Ok(batch)
        }

        async fn persist<DB>(&self, db: &DB, batch: &SqlStatementBatch) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            self.telemetry.record_engine_run(&self.orderbook_key);
            if self.fail {
                return Err(LocalDbError::CustomError(format!(
                    "apply failed {}",
                    self.orderbook_key
                )));
            }
            db.execute_batch(batch).await?;
            self.telemetry.record_persist();
            Ok(())
        }
    }

    #[derive(Clone)]
    struct StubStatusBus {
        telemetry: Telemetry,
        orderbook_key: String,
    }

    impl StubStatusBus {
        fn new(telemetry: Telemetry, orderbook_key: String) -> Self {
            Self {
                telemetry,
                orderbook_key,
            }
        }
    }

    #[async_trait(?Send)]
    impl StatusBus for StubStatusBus {
        async fn send(&self, message: &str) -> Result<(), LocalDbError> {
            self.telemetry.record_status(&self.orderbook_key, message);
            Ok(())
        }
    }

    type TestRunnerEnvironment = RunnerEnvironment<
        StubBootstrap,
        StubWindow,
        StubEvents,
        StubTokens,
        StubApply,
        StubStatusBus,
    >;

    type TestEngineBuilder = Arc<
        dyn Fn(
                &RunnerTarget,
            ) -> Result<
                EnginePipelines<
                    StubBootstrap,
                    StubWindow,
                    StubEvents,
                    StubTokens,
                    StubApply,
                    StubStatusBus,
                >,
                LocalDbError,
            > + Send
            + Sync,
    >;

    fn remote_url_a() -> Url {
        Url::parse("https://manifests.example/a.yaml").unwrap()
    }

    fn remote_url_b() -> Url {
        Url::parse("https://manifests.example/b.yaml").unwrap()
    }

    fn dump_url_a() -> Url {
        Url::parse("https://dumps.example/ob-a.sql").unwrap()
    }

    fn dump_url_b() -> Url {
        Url::parse("https://dumps.example/ob-b.sql").unwrap()
    }

    fn manifest_for_a() -> ManifestMap {
        make_manifest(remote_url_a(), ORDERBOOK_A, dump_url_a(), 111, "0xdead")
    }

    fn manifest_for_b() -> ManifestMap {
        make_manifest(remote_url_b(), ORDERBOOK_B, dump_url_b(), 222, "0xbeef")
    }

    fn manifest_for_both() -> ManifestMap {
        let mut map = manifest_for_a();
        map.extend(manifest_for_b());
        map
    }

    fn make_manifest(
        remote: Url,
        orderbook_address: Address,
        dump_url: Url,
        end_block: u64,
        end_hash: &str,
    ) -> ManifestMap {
        let manifest = LocalDbManifest {
            manifest_version: MANIFEST_VERSION,
            db_schema_version: DB_SCHEMA_VERSION,
            networks: HashMap::from([(
                NETWORK_KEY.to_string(),
                ManifestNetwork {
                    chain_id: CHAIN_ID,
                    orderbooks: vec![ManifestOrderbook {
                        address: orderbook_address,
                        dump_url,
                        end_block,
                        end_block_hash: Bytes::from_str(end_hash).unwrap(),
                        end_block_time_ms: 1_000,
                    }],
                },
            )]),
        };
        HashMap::from([(remote, manifest)])
    }

    fn two_orderbooks_settings_yaml() -> String {
        r#"
networks:
  anvil:
    rpcs:
      - https://rpc.example/anvil
    chain-id: 42161
subgraphs:
  anvil: https://subgraph.example/anvil
local-db-remotes:
  remote-a: https://manifests.example/a.yaml
  remote-b: https://manifests.example/b.yaml
local-db-sync:
  anvil:
    batch-size: 10
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 1
    finality-depth: 12
orderbooks:
  ob-a:
    address: 0x00000000000000000000000000000000000000a1
    network: anvil
    subgraph: anvil
    local-db-remote: remote-a
    deployment-block: 123
  ob-b:
    address: 0x00000000000000000000000000000000000000b2
    network: anvil
    subgraph: anvil
    local-db-remote: remote-b
    deployment-block: 456
"#
        .to_string()
    }

    fn single_orderbook_settings_yaml() -> String {
        r#"
networks:
  anvil:
    rpcs:
      - https://rpc.example/anvil
    chain-id: 42161
subgraphs:
  anvil: https://subgraph.example/anvil
local-db-remotes:
  remote-a: https://manifests.example/a.yaml
local-db-sync:
  anvil:
    batch-size: 10
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 1
    finality-depth: 12
orderbooks:
  ob-a:
    address: 0x00000000000000000000000000000000000000a1
    network: anvil
    subgraph: anvil
    local-db-remote: remote-a
    deployment-block: 123
"#
        .to_string()
    }

    fn prepare_db_baseline(db: &RecordingDb) {
        let tables: Vec<TableResponse> = REQUIRED_TABLES
            .iter()
            .map(|&name| TableResponse {
                name: name.to_string(),
            })
            .collect();
        db.set_json_value(&fetch_tables_stmt(), &tables);

        let metadata_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        db.set_json_value(&fetch_db_metadata_stmt(), &[metadata_row]);
        db.set_json_raw(&fetch_target_watermark_stmt(0, Address::ZERO), json!([]));
    }

    fn prepare_db_for_targets(db: &RecordingDb, targets: &[RunnerTarget]) {
        prepare_db_baseline(db);
        for target in targets {
            db.set_json_raw(
                &fetch_target_watermark_stmt(
                    target.inputs.target.chain_id,
                    target.inputs.target.orderbook_address,
                ),
                json!([]),
            );
            db.set_json_raw(
                &fetch_store_addresses_stmt(
                    target.inputs.target.chain_id,
                    target.inputs.target.orderbook_address,
                ),
                json!([]),
            );
        }
    }

    fn engine_builder_for_behaviors(
        telemetry: Telemetry,
        behaviors: HashMap<String, EngineBehavior>,
    ) -> TestEngineBuilder {
        let behaviors = Arc::new(behaviors);
        Arc::new(move |target: &RunnerTarget| {
            let behavior = behaviors
                .get(&target.orderbook_key)
                .copied()
                .unwrap_or(EngineBehavior::Success);
            let telemetry = telemetry.clone();
            telemetry.record_builder_init();
            let fail_apply = behavior == EngineBehavior::ApplyFail;
            let bootstrap = StubBootstrap::new(telemetry.clone(), target.orderbook_key.clone());
            let window = StubWindow::new(0, target.inputs.cfg.deployment_block);
            let events = StubEvents::new(target.inputs.cfg.deployment_block);
            let apply = StubApply::new(telemetry.clone(), target.orderbook_key.clone(), fail_apply);
            let status = StubStatusBus::new(telemetry.clone(), target.orderbook_key.clone());
            Ok(EnginePipelines::new(
                bootstrap, window, events, StubTokens, apply, status,
            ))
        })
    }

    fn build_environment(
        manifest_map: ManifestMap,
        behaviors: HashMap<String, EngineBehavior>,
        manifest_limit: usize,
        dump_limit: usize,
        telemetry: Telemetry,
    ) -> TestRunnerEnvironment {
        let manifest_arc = Arc::new(manifest_map);

        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    let prev = telemetry.record_manifest_fetch();
                    if prev >= manifest_limit {
                        panic!("manifest fetch called more than expected");
                    }
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = {
            let telemetry = telemetry.clone();
            let counter = Arc::new(AtomicUsize::new(0));
            Arc::new(move |url: &Url| {
                let telemetry = telemetry.clone();
                let counter = Arc::clone(&counter);
                let url = url.clone();
                Box::pin(async move {
                    let position = counter.fetch_add(1, Ordering::SeqCst);
                    if position >= dump_limit {
                        panic!("dump downloader invoked more than expected");
                    }
                    telemetry.record_dump(url.clone());
                    Ok(format!("-- dump for {}", url))
                }) as DumpFuture
            })
        };

        let engine_builder = engine_builder_for_behaviors(telemetry, behaviors);

        RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder)
    }

    fn expect_orderbooks(outcomes: &[SyncOutcome], expected: &[Address]) {
        let mut addrs: Vec<Address> = outcomes
            .iter()
            .map(|o| o.target.orderbook_address)
            .collect();
        addrs.sort();
        let mut expected_sorted = expected.to_vec();
        expected_sorted.sort();
        assert_eq!(addrs, expected_sorted);
    }

    #[test]
    fn with_environment_propagates_settings_parse_error() {
        let telemetry = Telemetry::default();
        let environment = build_environment(ManifestMap::new(), HashMap::new(), 1, 1, telemetry);

        let err = match ClientRunner::with_environment(
            "invalid: [yaml".to_string(),
            environment,
            AlwaysLeadership,
        ) {
            Ok(_) => panic!("invalid yaml should fail"),
            Err(err) => err,
        };

        assert!(matches!(
            err,
            LocalDbError::SettingsYaml(_) | LocalDbError::YamlScan(_)
        ));
    }

    #[test]
    fn default_environment_builds_engine_for_target() {
        let settings = single_orderbook_settings_yaml();
        let runner = ClientRunner::new(settings).expect("runner builds with default env");
        assert_eq!(runner.base_targets.len(), 1);

        let target = runner.base_targets[0].clone();
        let pipelines = runner
            .environment
            .build_engine(&target)
            .expect("default builder constructs engine");

        pipelines.into_engine();
    }

    #[tokio::test]
    async fn first_run_fetches_manifests_bootstraps_and_runs_engines() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcomes = runner.run(&db).await.expect("run succeeds");

        assert_eq!(outcomes.len(), 2);
        expect_orderbooks(&outcomes, &[ORDERBOOK_A, ORDERBOOK_B]);
        assert!(runner.manifests_loaded);
        assert!(runner.has_bootstrapped);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.engine_runs().len(), 2);
        assert_eq!(telemetry.persist_count(), 2);
        assert_eq!(telemetry.builder_inits(), 2);
        let records = telemetry.bootstrap_records();
        assert_eq!(records.len(), 2);
        assert!(records.iter().all(|record| record
            .dump_sql
            .as_ref()
            .is_some_and(|sql| sql.starts_with("-- dump for "))));
        assert_eq!(db.batch_calls().len(), 2);
    }

    #[tokio::test]
    async fn second_run_reuses_cached_manifest_and_skips_downloads() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db_first = RecordingDb::default();
        prepare_db_for_targets(&db_first, &runner.base_targets);
        runner.run(&db_first).await.expect("initial run succeeds");

        let db_second = RecordingDb::default();
        prepare_db_for_targets(&db_second, &runner.base_targets);
        let outcomes = runner.run(&db_second).await.expect("second run succeeds");

        assert_eq!(outcomes.len(), 2);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.builder_inits(), 4);
    }

    #[tokio::test]
    async fn leadership_guard_reused_across_runs() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership = CountingLeadership::new();
        let mut runner =
            ClientRunner::with_environment(settings, environment, leadership.clone()).unwrap();

        let db_first = RecordingDb::default();
        prepare_db_for_targets(&db_first, &runner.base_targets);
        runner.run(&db_first).await.expect("first run succeeds");

        let db_second = RecordingDb::default();
        prepare_db_for_targets(&db_second, &runner.base_targets);
        runner.run(&db_second).await.expect("second run succeeds");

        assert_eq!(
            leadership.call_count(),
            1,
            "leadership should be acquired only once per runner lifetime"
        );
    }

    #[tokio::test]
    async fn run_returns_empty_when_leadership_not_acquired() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership = SequenceLeadership::new(vec![LeadershipAction::Skip]);
        let mut runner = ClientRunner::with_environment(settings, environment, leadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let outcomes = runner.run(&db).await.expect("run succeeds");
        assert!(outcomes.is_empty());
        assert!(!runner.manifests_loaded);
        assert!(!runner.has_bootstrapped);
        assert_eq!(telemetry.manifest_fetch_count(), 0);
        assert!(telemetry.dump_requests().is_empty());
        assert_eq!(telemetry.builder_inits(), 0);
    }

    #[tokio::test]
    async fn leadership_skip_then_grant_fetches_once() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership =
            SequenceLeadership::new(vec![LeadershipAction::Skip, LeadershipAction::Grant]);
        let mut runner = ClientRunner::with_environment(settings, environment, leadership).unwrap();

        let db_skip = RecordingDb::default();
        prepare_db_baseline(&db_skip);
        let outcomes_skip = runner.run(&db_skip).await.expect("skip run succeeds");
        assert!(outcomes_skip.is_empty());
        assert!(!runner.manifests_loaded);
        assert!(!runner.has_bootstrapped);

        let db_grant = RecordingDb::default();
        prepare_db_for_targets(&db_grant, &runner.base_targets);
        let outcomes = runner.run(&db_grant).await.expect("second run succeeds");
        assert_eq!(outcomes.len(), 2);
        assert!(runner.manifests_loaded);
        assert!(runner.has_bootstrapped);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.builder_inits(), 2);
    }

    #[tokio::test]
    async fn leadership_error_is_propagated() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership =
            SequenceLeadership::new(vec![LeadershipAction::Fail("no leadership".into())]);
        let mut runner = ClientRunner::with_environment(settings, environment, leadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let err = runner.run(&db).await.expect_err("run should fail");
        matches!(err, LocalDbError::CustomError(message) if message == "no leadership");
        assert!(!runner.manifests_loaded);
        assert!(!runner.has_bootstrapped);
        assert_eq!(telemetry.manifest_fetch_count(), 0);
        assert!(telemetry.dump_requests().is_empty());
    }

    #[tokio::test]
    async fn provisioning_downloads_only_present_manifests() {
        let telemetry = Telemetry::default();
        let environment = build_environment(
            manifest_for_a(),
            HashMap::from([(ORDERBOOK_KEY_B.to_string(), EngineBehavior::Success)]),
            1,
            1,
            telemetry.clone(),
        );
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcomes = runner.run(&db).await.expect("run succeeds");
        assert_eq!(outcomes.len(), 2);
        let dumps = telemetry.dump_requests();
        assert_eq!(dumps.len(), 1);
        assert_eq!(dumps[0], dump_url_a());

        let records = telemetry.bootstrap_records();
        let record_a = records
            .iter()
            .find(|r| r.orderbook_key == ORDERBOOK_KEY_A)
            .expect("record for ob-a");
        assert!(record_a
            .dump_sql
            .as_ref()
            .is_some_and(|sql| sql.contains("-- dump for")));

        let record_b = records
            .iter()
            .find(|r| r.orderbook_key == ORDERBOOK_KEY_B)
            .expect("record for ob-b");
        assert!(record_b.dump_sql.is_none());
    }

    #[tokio::test]
    async fn run_returns_immediately_when_no_targets() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(ManifestMap::new(), HashMap::new(), 1, 0, telemetry.clone());
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        runner.base_targets.clear();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let outcomes = runner.run(&db).await.expect("run succeeds");
        assert!(outcomes.is_empty());
        assert!(runner.has_bootstrapped);
        assert_eq!(telemetry.builder_inits(), 0);
        assert_eq!(telemetry.dump_requests().len(), 0);
    }

    #[tokio::test]
    async fn bootstrap_failure_leaves_runner_unbootstrapped() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_a(), HashMap::new(), 2, 1, telemetry.clone());
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let failing_db = RecordingDb::default();
        prepare_db_baseline(&failing_db);
        failing_db.set_json_error(&fetch_tables_stmt(), LocalDbQueryError::database("boom"));

        let err = runner.run(&failing_db).await.expect_err("run should fail");
        matches!(
            err,
            LocalDbError::LocalDbQueryError(LocalDbQueryError::Database { .. })
        );
        assert!(!runner.has_bootstrapped);
        assert!(!runner.manifests_loaded);
        assert!(telemetry.dump_requests().is_empty());

        let success_db = RecordingDb::default();
        prepare_db_for_targets(&success_db, &runner.base_targets);
        runner.run(&success_db).await.expect("retry succeeds");
        assert!(runner.has_bootstrapped);
        assert_eq!(telemetry.manifest_fetch_count(), 2);
        assert_eq!(telemetry.dump_requests().len(), 1);
    }

    #[tokio::test]
    async fn manifest_fetch_failure_leaves_state_unloaded() {
        let telemetry = Telemetry::default();
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                Box::pin(async move {
                    telemetry.record_manifest_fetch();
                    Err(LocalDbError::CustomError("manifest boom".into()))
                }) as ManifestFuture
            })
        };

        let dump_downloader = Arc::new(|_url: &Url| {
            Box::pin(async move {
                Err(LocalDbError::CustomError(
                    "dump downloader should not run".into(),
                ))
            }) as DumpFuture
        });

        let engine_builder = engine_builder_for_behaviors(telemetry.clone(), HashMap::new());
        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let err = runner.run(&db).await.expect_err("run should fail");
        matches!(err, LocalDbError::CustomError(message) if message == "manifest boom");

        assert!(!runner.manifests_loaded);
        assert!(!runner.has_bootstrapped);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert!(telemetry.dump_requests().is_empty());
        assert_eq!(telemetry.builder_inits(), 0);
    }

    #[tokio::test]
    async fn dump_download_failure_is_propagated() {
        let telemetry = Telemetry::default();
        let manifest_map = manifest_for_a();
        let manifest_arc = Arc::new(manifest_map);
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    let prev = telemetry.record_manifest_fetch();
                    assert!(prev < 1, "manifest fetched too many times");
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = {
            let telemetry = telemetry.clone();
            Arc::new(move |url: &Url| {
                let telemetry = telemetry.clone();
                let url = url.clone();
                Box::pin(async move {
                    telemetry.record_dump(url.clone());
                    Err(LocalDbError::CustomError("download failed".into()))
                }) as DumpFuture
            })
        };

        let behaviors = HashMap::new();
        let engine_builder = engine_builder_for_behaviors(telemetry.clone(), behaviors);
        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);

        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let err = runner.run(&db).await.expect_err("run should fail");
        matches!(err, LocalDbError::CustomError(message) if message == "download failed");
        assert!(!runner.has_bootstrapped);
        assert!(!runner.manifests_loaded);
        assert_eq!(telemetry.dump_requests().len(), 1);
    }

    #[tokio::test]
    async fn engine_builder_error_is_propagated() {
        let telemetry = Telemetry::default();
        let manifest = manifest_for_both();
        let settings = two_orderbooks_settings_yaml();
        let behaviors = HashMap::from([(ORDERBOOK_KEY_B.to_string(), EngineBehavior::Success)]);

        let manifest_arc = Arc::new(manifest);
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    let prev = telemetry.record_manifest_fetch();
                    assert!(prev < 1, "manifest fetched too many times");
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = {
            let telemetry = telemetry.clone();
            let counter = Arc::new(AtomicUsize::new(0));
            Arc::new(move |url: &Url| {
                let telemetry = telemetry.clone();
                let counter = Arc::clone(&counter);
                let url = url.clone();
                Box::pin(async move {
                    let prev = counter.fetch_add(1, Ordering::SeqCst);
                    assert!(prev < 2, "dump downloaded too many times");
                    telemetry.record_dump(url.clone());
                    Ok(format!("-- dump for {}", url))
                }) as DumpFuture
            })
        };

        let engine_builder: TestEngineBuilder = {
            let telemetry = telemetry.clone();
            let behaviors = Arc::new(behaviors);
            Arc::new(move |target: &RunnerTarget| {
                if target.orderbook_key == ORDERBOOK_KEY_A {
                    return Err(LocalDbError::CustomError("builder failed".into()));
                }
                let behaviors = Arc::clone(&behaviors);
                let telemetry = telemetry.clone();
                let behavior = behaviors
                    .get(&target.orderbook_key)
                    .copied()
                    .unwrap_or(EngineBehavior::Success);
                telemetry.record_builder_init();
                let bootstrap = StubBootstrap::new(telemetry.clone(), target.orderbook_key.clone());
                let window = StubWindow::new(0, target.inputs.cfg.deployment_block);
                let events = StubEvents::new(target.inputs.cfg.deployment_block);
                let apply = StubApply::new(
                    telemetry.clone(),
                    target.orderbook_key.clone(),
                    behavior == EngineBehavior::ApplyFail,
                );
                let status = StubStatusBus::new(telemetry.clone(), target.orderbook_key.clone());
                Ok(EnginePipelines::new(
                    bootstrap, window, events, StubTokens, apply, status,
                ))
            })
        };

        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let err = runner.run(&db).await.expect_err("run should fail");
        matches!(err, LocalDbError::CustomError(message) if message == "builder failed");
        assert!(!runner.has_bootstrapped);
        assert_eq!(telemetry.dump_requests().len(), 2);
        let engine_runs = telemetry.engine_runs();
        assert!(
            engine_runs.iter().all(|key| key != ORDERBOOK_KEY_A),
            "engine should not run for target with builder failure"
        );
    }

    #[tokio::test]
    async fn engine_run_failure_is_propagated() {
        let telemetry = Telemetry::default();
        let mut behaviors = HashMap::new();
        behaviors.insert(ORDERBOOK_KEY_A.to_string(), EngineBehavior::ApplyFail);
        behaviors.insert(ORDERBOOK_KEY_B.to_string(), EngineBehavior::Success);
        let environment =
            build_environment(manifest_for_both(), behaviors, 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let err = runner.run(&db).await.expect_err("run should fail");
        matches!(err, LocalDbError::CustomError(message) if message.starts_with("apply failed"));
        assert!(!runner.has_bootstrapped);
        assert!(!runner.manifests_loaded);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert!(telemetry
            .engine_runs()
            .iter()
            .any(|key| key == ORDERBOOK_KEY_A));
    }

    #[tokio::test]
    async fn existing_dump_not_overwritten_without_manifest_entry() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(ManifestMap::new(), HashMap::new(), 1, 0, telemetry.clone());
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        assert_eq!(runner.base_targets.len(), 1);
        runner.base_targets[0].inputs.dump_str = Some("-- preloaded dump".to_string());

        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcomes = runner.run(&db).await.expect("run succeeds");
        assert_eq!(outcomes.len(), 1);
        assert!(telemetry.dump_requests().is_empty());
        let records = telemetry.bootstrap_records();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.orderbook_key, ORDERBOOK_KEY_A);
        assert_eq!(record.dump_sql.as_deref(), Some("-- preloaded dump"));
    }
}
