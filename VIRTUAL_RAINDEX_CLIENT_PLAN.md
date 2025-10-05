# Virtual Raindex Client Sync Plan

## Goal
- Keep each configured orderbook’s virtual Raindex state synchronized with locally stored chain events, expose dual (RPC + virtual) quotes via `RaindexOrder`, and provide UI/analysis hooks for real-time comparison and historical replay.

## Milestone 0 — Virtual Raindex Readiness
**Objective**: unblock the client registry work by making `virtual-raindex` wasm-safe, resumable from persisted state, and easy to hydrate with stored bytecode.

**Status**: wasm compatibility confirmed. `virtual-raindex` already compiles and tests cleanly with `--target wasm32-unknown-unknown`. No code changes required in the host for this PR. Remaining follow-up is to wire the wasm check into CI/prep scripts when convenient.

### 0.1 Wasm Compatibility Audit
- Run `nix develop -c cargo check --target wasm32-unknown-unknown -p virtual-raindex` to capture current compile blockers; record errors in `VIRTUAL_RAINDEX_IMPLEMENTATION.md`.
*No further work planned for Milestone 0.1.*

### 0.2 Snapshot Ergonomics
- Implement `VirtualRaindex::from_snapshot` (mirroring current `into_snapshot`) inside `crates/virtual-raindex/src/state.rs`, ensuring env/order/vault/store data copy by value so wasm builds avoid unsafe pointers.
- Add helper structs (`SnapshotBundle`, etc.) that package env metadata, decimals, and cache handles for transport; include serde derives for browser persistence.
- Provide a convenience loader under `crates/virtual-raindex/src/engine/mod.rs` to rebuild the engine from the bundle plus a ready cache.
- Document snapshot usage patterns in `VIRTUAL_RAINDEX_IMPLEMENTATION.md` and expose a `virtual_raindex::snapshot` module in `src/lib.rs`.
  - Snapshot helpers now live in `virtual_raindex::snapshot::{SnapshotBundle, CacheHandles}` with `VirtualRaindex::from_snapshot_bundle` performing cache validation.

### 0.3 Bytecode Cache Ingestion Helpers
- Add public constructors on `StaticCodeCache` for `(interpreter_addr, store_addr, bytecode_bytes)` so the client can hydrate from DB snapshots.
- Implement an ingestion helper that accepts order-level bytecode (`evaluable`) and memoises by hash/address; ensure we dedupe per orderbook.
- Expose a typed error enum covering missing bytecode, invalid encoding, and cache collisions; surface it to the sync pipeline docs.
- Update `VIRTUAL_RAINDEX_CLIENT_PLAN.md` breadcrumbs with the new helper locations once landing.
  - Cache ingestion lives in `virtual_raindex::StaticCodeCache::{from_artifacts, ingest_interpreters, ingest_stores}`; `StaticCodeCache::upsert_*` now returns `Result` with `RaindexError::InvalidBytecodeEncoding`/`BytecodeCollision` for error reporting.

### 0.4 Testing & Validation
- Add round-trip tests for snapshot export/import and cache hydration in `crates/virtual-raindex/src/engine/tests.rs`.
- Gate wasm-specific tests behind `--cfg wasm_test` and run them via `nix develop -c cargo test --target wasm32-unknown-unknown -p virtual-raindex --no-default-features --features web` once the feature flag exists.
- Ensure CI treats wasm checks as required (non-optional) and document local run commands in `VIRTUAL_RAINDEX_IMPLEMENTATION.md`.
  - Snapshot/cache tests landed in `crates/virtual-raindex/src/engine/tests.rs`; wasm coverage uses `#[cfg(wasm_test)]` with commands documented in `VIRTUAL_RAINDEX_IMPLEMENTATION.md`.

### Exit Criteria Checklist
- `nix develop -c cargo check --target wasm32-unknown-unknown -p virtual-raindex` passes without feature toggles.
- `VirtualRaindex::from_snapshot` rebuilds state equivalent to replaying the same events (asserted in tests).
- Cache ingestion helpers populate `StaticCodeCache` from raw bytecode blobs without RPC calls.
- CI workflow updated with wasm target, and docs reflect the new commands.

## Milestone 1 — Live Virtual Raindex Sync Core
**Objective**: land a reusable "live" wrapper inside `virtual-raindex` so any host can keep the engine current while tolerating missing bytecode and persisting state.
- **SyncEngine trait in `virtual-raindex`**: define the trait under a new `virtual_raindex::live` module. It should expose async methods for discovering new mutations, returning already-hydrated bytecode, and reporting any artifacts still fetching so callers can surface partial readiness instead of blocking.
- **`LiveVirtualRaindex` facade**: add a struct that owns a `VirtualRaindex` instance, a `SyncEngine`, and cache handles. It coordinates snapshot restore, mutation application, and ensures read-only snapshots remain available while writes are serialised per orderbook.
- **Non-blocking bytecode strategy**: design the engine so it schedules RPC fetches for unseen interpreters/stores in the background, tracks pending artifacts, and retries queued mutations once cache entries arrive. Emit advisory status that upper layers can convert into "warming up" UI states.
- **Persistence adapters**: surface lightweight traits for snapshot storage, cursor tracking, and optional metrics/logging. Provide an in-memory adapter for tests and document how hosts plug in sqlite/Postgres or other storage later.
- **Stub implementation + fixtures**: ship a `StubSyncEngine` + sample data set that exercises the live facade without needing the local DB. Include hooks for deterministic outcomes so higher-layer tests can rely on it immediately.
- **Tests**: unit tests validating partial mutation application, background bytecode completion, snapshot round-trips via the new adapters, and concurrency behaviour (readers vs. mutating sync calls). Integration smoke test using the stub engine to prove the live facade can stream quotes while bytecode fetches resolve.
- **Implementation notes**: document the internal state machine (`Idle` → `Syncing` → `PendingArtifacts`) and concurrency strategy (`tokio::sync` primitives for native, `wasm-bindgen-futures`/`gloo_timers` shims for WASM) in `docs/virtual_raindex_live.md`. Capture how bytecode fetch tasks enqueue retries (e.g., via `tokio::task::JoinSet`) and the shape of telemetry events so downstream hosts can listen for "warming up" advisories.

## Milestone 2 — Local DB → Virtual State Pipeline *(Deferred)*
**Status**: Blocked until the local-db raw events + cache tables merge lands; revisit after the live sync core lands and the database schema is available.
- **Share CLI utilities**: extract the existing CLI sync helpers (event decoding, token prep, SQL builders) into shared modules under `rain_orderbook_common` so the client and CLI reuse the same logic.
- **Event extraction**: add query helpers under `raindex_client::local_db::query` that stream `raw_events` for a given orderbook, ordered by `(block_number, log_index)`, and recover the stored `raw_json` payloads. Reuse `LocalDb::decode_events` to materialize typed events from those JSON blobs, then collate higher-level records from the specialised tables (`order_events`, `take_orders`, `clear_v3_events`, `after_clear_v2_events`, `interpreter_store_sets`) so each block/log yields a single `OrderBookEvent` or `StoreEvent`. Join `ClearV3` with the matching `AfterClearV2`, and attach token decimals from `erc20_tokens` when available.
- **Mutation builder**: using the extracted events, invoke the existing `virtual_raindex::events::{orderbook, store}` converters to obtain `RaindexMutation` batches. Introduce a sync driver that keeps per-orderbook cursors (block number, log index, UTC timestamp), hydrates token decimals into the virtual state, applies mutations, and updates both the virtual `Env` and the persisted cursor.
- **Bytecode hydration & persistence**: create a new local DB table (e.g., `bytecode_artifacts`) to persist interpreter/store bytecode with columns `(address, kind, bytecode, fetched_at_block, fetched_at_ts)`. When the mutation builder encounters a new interpreter/store address, look it up in the table; if missing or stale, call `eth_getCode` via the configured `RpcClient`, persist the result, and feed it into `StaticCodeCache`. Orders carry their evaluable bytecode via `order_events.order_bytes`, so decode once per order and memoize it alongside the cache entry. Provide a loader that rehydrates `StaticCodeCache` directly from the database before replaying mutations.
- **Tests**: unit tests covering raw → decoded event reconstruction (including clear pairings), mutation translation, cursor persistence, and bytecode cache storage; integration test with an in-memory sqlite database that seeds sampled events, runs the sync driver, and verifies the resulting `VirtualRaindex` snapshot/env + cache contents.

## Milestone 3 — Virtual Raindex Registry in Client
- **Adopt the live core**: wrap `virtual_raindex::live::LiveVirtualRaindex` inside a client-facing registry so each `(chain_id, orderbook)` reuses the shared engine, cache, and snapshot helpers from Milestone 1.
- **Client sync adapters**: provide a thin adapter that plugs the Milestone 1 `StubSyncEngine` into the registry for now, and leave shims so the Milestone 2 local-db engine can be swapped in without touching callers.
- **Registry scaffolding**: implement a shared `VirtualRaindexRegistry` (or similarly named service) within the client that
  - manages one `VirtualRaindex` instance per `(chain_id, orderbook_address)`;
  - owns the bytecode cache loader, sync cursors, and snapshot persistence/restoration using the helpers from Milestone 0;
  - lazily initialises instances from persisted snapshots (stored in a new `virtual_snapshots` table holding blob + metadata such as block, timestamp, checksum), applying catch-up mutations before releasing them for use;
  - exposes async methods to acquire a handle (read-only or mutable) while serialising mutation application per orderbook, but allowing read-only quotes from clones/snapshots;
  - records registry metrics/errors so the UI can surface “virtual state stale” warnings.
- **Hook placeholder data**: allow hard-coded fixtures or RPC-sourced slices (fed through the stub engine) so we can smoke-test registry lifecycle, quote comparisons, and UI plumbing without waiting on the DB merge.
- **Tests**: unit tests ensuring the registry reuses instances across orders, rehydrates from snapshots, updates cursors atomically, and serialises concurrent mutation requests; add dedicated tests for the stub integration to guarantee deterministic outputs and make it easy to swap for mocks (trait object) in Milestone 4 integration tests.

## Milestone 4 — Raindex Client API Surface
- Add a `RaindexOrder` method (e.g., `ensure_virtual_sync_and_quote`) that talks to the registry, triggers sync as needed, returns both RPC and virtual quotes, and reports metadata (latest block, timestamp, applied events, cache status, snapshot age).
- Handle edge cases: missing decimals, absent bytecode, divergent snapshots. Provide descriptive errors for UI consumption.
- **Tests**: unit tests mocking the registry; integration test wiring the sqlite fixture through the real virtual engine to assert quote parity on known scenarios and verify error propagation.

## Milestone 5 — Webapp Comparison UI
- Inject the new API into the quoting flow, displaying RPC and virtual quotes side-by-side with freshness indicators and surfaced warnings from the registry.
- **Tests**: component tests (Vitest/React Testing Library) covering loading/error/compare states; optional Playwright story to verify end-to-end interaction.

## Milestone 6 — Historical Replay & Visualization
- Implement replay tooling that advances block/time in one-second steps, replays events in order via the registry, captures quotes, and records event annotations.
- Expose the replay output to the webapp and render a chart of quote history with event markers and trade annotations.
- **Tests**: deterministic replay test using recorded fixtures; unit tests for data transformation → chart props; UI snapshot ensuring chart renders with sample data.

## Cross-Cutting Tasks
- Ensure token decimals are always available (fallback to ERC-20 metadata service if local DB misses them).
- Document registry + sync flows so other consumers (CLI, tests) can reuse the pipeline.
- Track cache invalidation (last synced block per orderbook) alongside event ingestion to support incremental updates and optional backfill.
- Add logging/metrics hooks (behind feature flags) for sync duration, cache misses, bytecode fetches, snapshot loads, and quote divergence.
- Capture telemetry around registry concurrency (queue length, wait times) to help diagnose UI latency.

## Breadcrumbs

### Milestone 0
- `crates/virtual-raindex/src/host/revm.rs`, `Cargo.toml`, and `src/lib.rs` for wasm gating and host exports.
- `crates/virtual-raindex/src/state.rs`, `src/engine/mod.rs` for snapshot creation; add `from_snapshot` nearby.
- `crates/virtual-raindex/src/cache.rs` for cache ingestion helpers; align tests in `src/engine/tests.rs` and `src/integration_tests.rs`.
- CI setup lives under `.github/workflows/` and `flake.nix`/`pref-all.sh` for wasm build steps.

### Milestone 1
- New live sync core under `crates/virtual-raindex/src/live/` (e.g., `sync_engine.rs`, `mod.rs`, `live_virtual_raindex.rs`).
- Persistence adapters alongside `crates/virtual-raindex/src/live/persistence.rs` with an in-memory test harness in `src/live/tests.rs`.
- Background bytecode fetch helpers wired through `crates/virtual-raindex/src/cache.rs` and re-exported via `virtual_raindex::live`.
- State machine + concurrency notes tracked in `docs/virtual_raindex_live.md` (write during implementation).

### Milestone 2
- Local DB schema & helpers: `crates/common/src/raindex_client/local_db/query/create_tables/query.sql`, `query/fetch_*`, `local_db/decode.rs`, `local_db/sync.rs`.
- CLI sync utilities to share: `crates/cli/src/commands/local_db/sync/**/*` and `crates/cli/src/commands/local_db/decode_events.rs`.
- RPC client & settings: `crates/common/src/raindex_client/rpc_client.rs`, `crates/settings` for orderbook configs.
- Virtual mutation converters: `crates/virtual-raindex/src/events/orderbook.rs` and `events/store.rs`.
- Add new persistence tables alongside existing migrations in `crates/common/src/raindex_client/local_db/query/create_tables/`.
- Test fixtures: `crates/test_fixtures/`, `crates/virtual-raindex/src/integration_tests.rs` for sample mutations.

### Milestone 3
- Registry module under `crates/common/src/raindex_client/virtual/` (create) with re-exports in `crates/common/src/raindex_client/mod.rs`.
- Snapshot storage tables to extend local-db schema (`create_tables/query.sql`).
- Reference implementations for wiring `LiveVirtualRaindex` within the client, including stub adapters in `crates/common/src/raindex_client/virtual/stub.rs`.

### Milestone 4
- `crates/common/src/raindex_client/order.rs` (or equivalent) for `RaindexOrder` methods; check `crates/common/src/raindex_client/mod.rs` for exports.
- RPC quoting lives in `crates/quote` and `crates/common/src/raindex_client/quote.rs` – ensure parity.
- Reuse registry hooks added in Milestone 3.

### Milestone 5
- Webapp quoting components: `packages/webapp/src/features/orders/` and API hooks in `packages/webapp/src/hooks/`.
- Shared client code via `packages/webapp/src/lib/raindexClient` (confirm actual path).
- UI state management (Redux/Zustand) under `packages/webapp/src/state/`.

### Milestone 6
- Replay utilities can live under `crates/common/src/raindex_client/replay` or shared crate; use timeline components in `packages/webapp/src/features/charts/`.
- Historical event fixtures in `test-resources/` or `crates/test_fixtures`; extend CLI history tools if needed.

### Cross-Cutting
- Metrics/logging hooks: review `crates/common/src/analytics` (if present) and `packages/webapp/src/lib/logging`.
- Orderbook settings and token metadata caching: `crates/settings`, `crates/common/src/raindex_client/local_db/token_fetch.rs`.
- Concurrency patterns: check existing async primitives in `crates/common/src/async_utils` (if present) or use `tokio::sync` in WASM via `wasm-bindgen-futures`.
