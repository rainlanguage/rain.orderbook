# Live Virtual Raindex Overview

The live sync facade lifts the single-threaded `VirtualRaindex` engine into an async-friendly runner that
can tolerate partially hydrated caches, stream new mutations, and persist snapshots between polls.

## State Machine
- **Idle** – no outstanding work; readers see the latest snapshot and caches are warm.
- **Syncing** – at least one mutation batch applied or new bytecode cached in the last poll.
- **PendingArtifacts** – queued mutations are waiting for interpreter/store bytecode; UI should surface a
  "warming up" state.
- **Errored** – a fatal cache/mutation failure occurred; surface telemetry and halt further polling until
  operators intervene.

`LiveVirtualRaindex::sync_once` transitions between these phases by combining SyncEngine output with the
`BytecodeWarmupQueue`. When fresh bytecode lands the queue is drained and deferred mutations are retried
without blocking read access.

## Core Components
- `live::SyncEngine` – host-supplied async trait that yields mutation envelopes, hydrated bytecode, and
  currently fetching artifacts. The stub implementation lives in `live::stub` and is backed by deterministic
  scripts.
- `live::LiveVirtualRaindex` – façade owning the engine, code cache, interpreter host, and persistence
  adapters. The builder requires:
  - `Arc<dyn SnapshotStore>` – async load/save of `SnapshotBundle` values.
  - `Arc<dyn CursorStore<C>>` – per-orderbook cursor persistence for the engine-specific cursor type.
  - Optional `Arc<dyn MetricsSink>` – receives `LiveStatus` snapshots and `SyncProgress` counters after
    each poll.
- `live::BytecodeWarmupQueue` – tracks deferred mutation envelopes alongside the `ArtifactId`s they require.
  Once `LiveCodeCache::is_ready` reports all artifacts cached, the envelope is reapplied automatically.
- `live::cache_ext::LiveCodeCache` – small extension trait for caches that can ingest bytecode artifacts and
  report readiness (implemented for `StaticCodeCache`).

### Adapter Traits
- `SnapshotStore` implementations load and persist `SnapshotBundle` instances keyed by orderbook address.
  They should be idempotent (persisting the same bundle twice is a no-op), return `Ok(None)` when no prior
  state exists, and surface durable-storage failures via `RaindexError` so callers can decide whether to retry
  or halt. The facade persists snapshots only after it applies at least one mutation batch.
- `CursorStore<C>` tracks the sync engine’s cursor type. Calls are per-orderbook, so stores can scope locking
  and transactional guarantees to a single key. Returning `Ok(None)` is treated as “start from genesis”; any
  error bubbles out of `sync_once` to avoid losing cursor position.
- `MetricsSink` receives best-effort status/progress updates after each poll. The default `NoopMetrics`
  implementation discards events, but production sinks could forward into tracing spans, logs, or be surfaced to the UI.
  Failures should be handled internally; the facade ignores errors so telemetry does not interrupt syncing.
- `InMemorySnapshotStore` and `InMemoryCursorStore` provide in-process baselines for tests/examples and can
  serve as blueprints for persistent implementations (e.g. sqlite/Postgres, sled, or browser storage).

## Persistence Flow
1. Load initial `SnapshotBundle` via builder-provided snapshot store or optional `with_initial_snapshot`.
2. Load the last cursor (if any) from the cursor store; blank instances persist an empty snapshot on boot to
   seed downstream storage.
3. After each successful poll, persist the new cursor and snapshot (only when mutations were applied) to keep
   offline resumes cheap.

## Bytecode Warmup Strategy
- Sync engines may stream `MutationEnvelope`s before matching bytecode is ready. The façade determines the
  required `ArtifactId`s from order mutations and queues them.
- `SyncEngine::poll` may also report currently fetching artifact IDs; these are merged with the queue when
  constructing the `LiveStatus` so callers can render granular loading hints.
- Once `LiveVirtualRaindex` ingests `BytecodeArtifact`s via `LiveCodeCache::ingest`, deferred mutations are
  retried and the status switches back to `Syncing`/`Idle` with a `Ready` advisory.

## Stub + Fixtures
- `StubSyncEngine::demo()` loads `test-resources/virtual_raindex/live/demo.json` and replays a deterministic
  script exercising env updates, deferred order mutations, bytecode hydration, and heartbeats.
- Additional scripts can be loaded via `StubSyncEngine::from_json_file` for integration tests or CLI demos.
- Each script step can inject `MutationEnvelope`s, raw `BytecodeArtifact`s (hex encoded), pending artifacts,
  and optional heartbeats to keep the facade active without replaying data.

## Usage Snippet
```rust
use std::sync::Arc;
use alloy::primitives::Address;
use virtual_raindex::live::{
    InMemoryCursorStore, InMemorySnapshotStore, LiveVirtualRaindex, NoopMetrics,
};
use virtual_raindex::{host::RevmInterpreterHost, StaticCodeCache};
use virtual_raindex::live::stub::StubSyncEngine;

let orderbook = Address::from([0x11u8; 20]);
let sync_engine = Arc::new(StubSyncEngine::demo());
let code_cache = Arc::new(StaticCodeCache::default());
let host = Arc::new(RevmInterpreterHost::new(code_cache.clone()));
let snapshots = Arc::new(InMemorySnapshotStore::new());
let cursors = Arc::new(InMemoryCursorStore::new());
let metrics = Arc::new(NoopMetrics);

let live = LiveVirtualRaindex::builder(orderbook, sync_engine, code_cache, host)
    .with_snapshot_store(snapshots)
    .with_cursor_store(cursors)
    .with_metrics(metrics)
    .build()
    .await?;

let status = live.sync_once().await?; // drive one poll
```

## Telemetry Considerations
- Status transitions are reported via `MetricsSink::record_status`; `SyncProgress` includes batch counts,
  cached artifacts, and queued mutation totals for latency tracking.
- Callers should surface `LiveStatus::pending.artifacts` when non-empty and consider gating quote streams
  until a `Ready` advisory arrives.

## Next Steps
- Wire production sync engines that source chain events/local DB snapshots.
- Extend fixtures with richer orders so web/client integration tests can cover full quote flows.
- Route `LiveStatus::Errored` through existing telemetry pipelines and expose health checks to the registry.
