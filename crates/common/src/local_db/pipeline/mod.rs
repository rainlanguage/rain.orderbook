//! Split‑traits API for the local DB sync engine.
//!
//! This module defines minimal, focused traits that partition sync
//! responsibilities into composable units. Environment‑specific behavior
//! (browser vs producer) is provided by implementing these traits; the core
//! orchestrator depends only on the traits and does not
//! branch on environment.
//!

pub mod adapters;

use crate::erc20::TokenInfo;
use crate::local_db::decode::{DecodedEvent, DecodedEventData};
use crate::local_db::query::{
    fetch_erc20_tokens_by_addresses::Erc20TokenRow, LocalDbQueryExecutor, SqlStatementBatch,
};
use crate::local_db::{FetchConfig, LocalDbError};
use crate::rpc_client::{BlockRange, LogEntryResponse};
use alloy::primitives::Address;
use async_trait::async_trait;
use url::Url;

/// Identifies the logical target (orderbook) for a sync cycle.
///
/// Multi‑tenant writes/reads are always keyed by this structure.
#[derive(Debug, Clone)]
pub struct TargetKey {
    /// Chain id for the orderbook deployment.
    pub chain_id: u64,
    /// Address of the orderbook contract.
    pub orderbook_address: Address,
}

/// Optional manual window overrides usually supplied by CLI/producer.
///
/// Orchestrators apply these after computing a finality‑clamped safe head.
#[derive(Debug, Clone, Default)]
pub struct WindowOverrides {
    /// Override the start block (inclusive). When omitted the watermark or
    /// deployment block is used per policy.
    pub start_block: Option<u64>,
    /// Override the end/target block (inclusive) before finality clamping.
    pub end_block: Option<u64>,
}

/// Finality policy for windowing.
///
/// The safe head is computed as `max(deployment_block, latest - depth)` and
/// used as the upper bound for the target block, preventing tailing too close
/// to the chain head.
#[derive(Debug, Clone)]
pub struct FinalityConfig {
    /// Safe head depth; 0 means “no finality buffer”.
    pub depth: u32,
}

/// Static configuration supplied to a sync cycle.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Block where the orderbook was deployed; the start block never goes
    /// below this.
    pub deployment_block: u64,
    /// Fetch configuration (batch sizes, concurrency, etc.).
    pub fetch: FetchConfig,
    /// Finality policy.
    pub finality: FinalityConfig,
    /// Optional manual window overrides (typically CLI only).
    pub window_overrides: WindowOverrides,
}

/// Coarse execution summary for a single sync cycle.
#[derive(Debug, Clone)]
pub struct SyncOutcome {
    /// Target that was synced.
    pub target: TargetKey,
    /// Start block (inclusive) that was used.
    pub start_block: u64,
    /// Target block (inclusive) that was used.
    pub target_block: u64,
    /// Count of raw logs fetched across orderbook and stores.
    pub fetched_logs: usize,
    /// Count of decoded events materialized during the cycle.
    pub decoded_events: usize,
}

/// Descriptor for a runner‑supplied seed dump to import during bootstrap.
///
/// The runner decides which dump to use (if any) by consulting a manifest
/// or other policy and passes a reference here. Implementations can choose
/// which artefact to consume (e.g., `.sql` vs `.sql.gz` vs copying `.db`).
#[derive(Debug, Clone)]
pub struct SeedDump {
    /// End block of the dump. Used for sanity checks and status messages.
    pub end_block: u64,
    /// Optional path to a plain SQL file on disk.
    pub sql_path: Option<String>,
    /// Optional path to a gzipped SQL file on disk.
    pub sql_gz_path: Option<String>,
    /// Optional path to a ready‑to‑use database file.
    pub db_path: Option<String>,
}

/// Ensures the database is ready for incremental sync and applies optional
/// data‑only seed dumps per environment policy.
///
/// Responsibilities (concrete):
/// - Ensure schema tables exist. Dumps must not include DDL.
/// - Version gate via `db_metadata` (read/init, fail/reset on mismatch per
///   environment policy).
///
/// Policy (environment‑specific examples):
/// - Browser: consult manifest; if DB is empty or sufficiently behind a
///   manifest dump, import a data‑only dump then continue incremental.
/// - Producer: initialize a per‑target DB from the latest dump at start of
///   run (fresh seed). If no dump, proceed with schema‑only and incremental.
#[async_trait(?Send)]
pub trait BootstrapPipeline {
    /// Executes bootstrap for `target` using the provided DB executor.
    ///
    /// The optional `seed_dump` is provided by the runner (already selected
    /// from a manifest or other policy). Implementations must ensure schema
    /// first, then import the seed if present, then finalize version gating.
    async fn run<DB>(
        &self,
        db: &DB,
        target: &TargetKey,
        seed_dump: Option<&SeedDump>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized;
}

/// Coarse‑grained progress/status publishing.
///
/// Keep messages short and stable; a richer typed snapshot can be layered on
/// top without changing the pipeline contracts.
#[async_trait(?Send)]
pub trait StatusBus {
    /// Publishes a human‑readable status message.
    async fn send(&self, message: &str) -> Result<(), LocalDbError>;
}

/// Computes the inclusive `[start_block, target_block]` for a cycle.
///
/// Responsibilities (concrete):
/// - Read watermark for the target (last synced block, and optionally last
///   block hash).
/// - Compute `safe_head = max(deployment_block, latest - finality.depth)` and
///   apply overrides from `cfg.window_overrides` (subject to clamp).
///
/// Policy (environment‑specific):
/// - Continuity check: producer verifies parent hash continuity vs stored
///   watermark hash; browser may skip or apply a light check.
///
/// Invariants:
/// - If `start_block > target_block`, the sync cycle is a no‑op.
#[async_trait(?Send)]
pub trait WindowPipeline {
    /// Returns `(start_block, target_block)` for the cycle.
    async fn compute<DB>(
        &self,
        db: &DB,
        target: &TargetKey,
        cfg: &SyncConfig,
        latest_block: u64,
    ) -> Result<(u64, u64), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized;
}

/// Facade over the event fetch + decode steps.
///
/// Responsibilities (concrete):
/// - Decode via shared ABI into stable `DecodedEventData<DecodedEvent>`.
/// - Provide a uniform surface for fetching orderbook/store logs.
///
/// Policy (environment‑specific):
/// - Backend selection: browser uses regular/public RPCs; producer uses
///   HyperRPC for logs and regular RPCs for tokens.
#[async_trait(?Send)]
pub trait EventsPipeline {
    /// Returns the latest chain block number according to the backend.
    async fn latest_block(&self) -> Result<u64, LocalDbError>;

    /// Fetches orderbook logs within the inclusive block range.
    async fn fetch_orderbook(
        &self,
        orderbook_address: Address,
        range: BlockRange,
        cfg: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError>;

    /// Fetches interpreter store logs for the supplied addresses.
    async fn fetch_stores(
        &self,
        store_addresses: &[Address],
        range: BlockRange,
        cfg: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError>;

    /// Decodes raw logs into typed events. Decoding must be deterministic
    /// for identical input logs.
    fn decode(
        &self,
        logs: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError>;
}

/// ERC‑20 token metadata lookup pipeline.
///
/// Responsibilities (concrete):
/// - Read existing token rows for a chain and compute the missing set.
/// - Fetch metadata for missing tokens and return typed values; SQL generation
///   is handled by the Apply pipeline.
///
/// Invariants:
/// - Upserts must be idempotent and keyed by `(chain_id, lower(address))`.
#[async_trait(?Send)]
pub trait TokensPipeline {
    /// Loads existing token rows for the provided lowercase addresses.
    async fn load_existing<DB>(
        &self,
        db: &DB,
        chain_id: u64,
        token_addrs_lower: &[String],
    ) -> Result<Vec<Erc20TokenRow>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized;

    /// Fetches metadata for missing tokens using the supplied RPC endpoints.
    async fn fetch_missing(
        &self,
        rpcs: &[Url],
        missing: Vec<Address>,
        cfg: &FetchConfig,
    ) -> Result<Vec<(Address, TokenInfo)>, LocalDbError>;
}

/// Translates fetched/decoded data into SQL and persists it atomically.
///
/// Responsibilities (concrete):
/// - Build a transactional batch containing:
///   - Raw events INSERTs.
///   - Token upserts for provided `(Address, TokenInfo)` pairs.
///   - Decoded event INSERTs for all orderbook‑scoped tables, binding the
///     target key.
///   - Watermark update to the `target_block` (and later last hash).
/// - Persist the batch with a single‑writer gate; must assert that the batch
///   is transaction‑wrapped and fail if not.
///
/// Policy (environment‑specific):
/// - Dump export after a successful persist (producer only); browser is no‑op.
#[async_trait(?Send)]
pub trait ApplyPipeline {
    /// Builds the SQL batch for a cycle. The batch must be suitable for
    /// atomic execution (the caller will ensure single‑writer semantics).
    fn build_batch(
        &self,
        target: &TargetKey,
        target_block: u64,
        raw_logs: &[LogEntryResponse],
        decoded_events: &[DecodedEventData<DecodedEvent>],
        tokens_to_upsert: &[(Address, TokenInfo)],
    ) -> SqlStatementBatch;

    /// Persists the previously built batch. Implementations must assert that
    /// the input is wrapped in a transaction and return an error otherwise.
    async fn persist<DB>(&self, db: &DB, batch: &SqlStatementBatch) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized;

    /// Optional policy hook to export dumps after a successful persist.
    /// Default implementation is a no‑op.
    async fn export_dump<DB>(
        &self,
        _db: &DB,
        _target: &TargetKey,
        _end_block: u64,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        Ok(())
    }
}
