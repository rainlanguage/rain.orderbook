use std::collections::HashSet;
use std::sync::Arc;

use alloy::primitives::Address;
use parking_lot::RwLock;

use crate::{
    error::Result, host::InterpreterHost, snapshot::SnapshotBundle, state::RaindexMutation,
    BytecodeKind, VirtualRaindex,
};

use super::{
    adapter::{
        CursorStore, InMemoryCursorStore, InMemorySnapshotStore, MetricsSink, SnapshotStore,
    },
    bytecode::BytecodeWarmupQueue,
    cache_ext::LiveCodeCache,
    status::{LiveStatus, SyncProgress},
    sync_engine::{ArtifactId, MutationEnvelope, SyncEngine},
};

pub struct LiveVirtualRaindex<E, C, H>
where
    E: SyncEngine,
    C: LiveCodeCache,
    H: InterpreterHost,
{
    orderbook: Address,
    sync_engine: Arc<E>,
    code_cache: Arc<C>,
    snapshot_store: Arc<dyn SnapshotStore>,
    cursor_store: Arc<dyn CursorStore<E::Cursor>>,
    metrics: Option<Arc<dyn MetricsSink>>,
    inner: RwLock<LiveInner<E::Cursor, C, H>>,
}

struct LiveInner<CURSOR, C, H>
where
    C: LiveCodeCache,
    H: InterpreterHost,
{
    raindex: VirtualRaindex<C, H>,
    cursor: Option<CURSOR>,
    status: LiveStatus,
    warmup_queue: BytecodeWarmupQueue,
}

impl<CURSOR, C, H> LiveInner<CURSOR, C, H>
where
    C: LiveCodeCache,
    H: InterpreterHost,
{
    fn new(raindex: VirtualRaindex<C, H>) -> Self {
        Self {
            raindex,
            cursor: None,
            status: LiveStatus::idle(),
            warmup_queue: BytecodeWarmupQueue::new(),
        }
    }
}

pub struct LiveVirtualRaindexBuilder<E, C, H>
where
    E: SyncEngine,
    C: LiveCodeCache,
    H: InterpreterHost,
{
    orderbook: Address,
    sync_engine: Arc<E>,
    code_cache: Arc<C>,
    interpreter_host: Arc<H>,
    snapshot_store: Option<Arc<dyn SnapshotStore>>,
    cursor_store: Option<Arc<dyn CursorStore<E::Cursor>>>,
    metrics: Option<Arc<dyn MetricsSink>>,
    initial_snapshot: Option<SnapshotBundle>,
}

impl<E, C, H> LiveVirtualRaindexBuilder<E, C, H>
where
    E: SyncEngine,
    C: LiveCodeCache,
    H: InterpreterHost,
{
    pub fn new(
        orderbook: Address,
        sync_engine: Arc<E>,
        code_cache: Arc<C>,
        interpreter_host: Arc<H>,
    ) -> Self {
        Self {
            orderbook,
            sync_engine,
            code_cache,
            interpreter_host,
            snapshot_store: None,
            cursor_store: None,
            metrics: None,
            initial_snapshot: None,
        }
    }

    pub fn with_snapshot_store(mut self, store: Arc<dyn SnapshotStore>) -> Self {
        self.snapshot_store = Some(store);
        self
    }

    pub fn with_cursor_store(mut self, store: Arc<dyn CursorStore<E::Cursor>>) -> Self {
        self.cursor_store = Some(store);
        self
    }

    pub fn with_metrics(mut self, metrics: Arc<dyn MetricsSink>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_initial_snapshot(mut self, snapshot: SnapshotBundle) -> Self {
        self.initial_snapshot = Some(snapshot);
        self
    }

    pub async fn build(self) -> Result<LiveVirtualRaindex<E, C, H>> {
        let snapshot_store: Arc<dyn SnapshotStore> = self
            .snapshot_store
            .unwrap_or_else(|| Arc::new(InMemorySnapshotStore::new()));
        let cursor_store: Arc<dyn CursorStore<E::Cursor>> = self
            .cursor_store
            .unwrap_or_else(|| Arc::new(InMemoryCursorStore::new()));

        if let Some(snapshot) = &self.initial_snapshot {
            snapshot_store.persist_snapshot(snapshot.clone()).await?;
        }

        let snapshot_bundle = match self.initial_snapshot {
            Some(snapshot) => Some(snapshot),
            None => snapshot_store.load_snapshot(self.orderbook).await?,
        };

        let cursor = cursor_store.load_cursor(self.orderbook).await?;

        let raindex = if let Some(bundle) = snapshot_bundle.clone() {
            VirtualRaindex::from_snapshot_bundle(
                bundle,
                self.code_cache.clone(),
                self.interpreter_host.clone(),
            )?
        } else {
            VirtualRaindex::new(
                self.orderbook,
                self.code_cache.clone(),
                self.interpreter_host.clone(),
            )
        };

        let baseline_snapshot = if snapshot_bundle.is_none() {
            Some(SnapshotBundle::from_snapshot(
                self.orderbook,
                raindex.snapshot(),
            ))
        } else {
            None
        };

        let mut inner = LiveInner::new(raindex);
        inner.cursor = cursor;

        if let Some(snapshot) = baseline_snapshot {
            snapshot_store.persist_snapshot(snapshot).await?;
        }

        Ok(LiveVirtualRaindex {
            orderbook: self.orderbook,
            sync_engine: self.sync_engine,
            code_cache: self.code_cache,
            snapshot_store,
            cursor_store,
            metrics: self.metrics,
            inner: RwLock::new(inner),
        })
    }
}

impl<E, C, H> LiveVirtualRaindex<E, C, H>
where
    E: SyncEngine,
    C: LiveCodeCache,
    H: InterpreterHost,
{
    pub fn builder(
        orderbook: Address,
        sync_engine: Arc<E>,
        code_cache: Arc<C>,
        interpreter_host: Arc<H>,
    ) -> LiveVirtualRaindexBuilder<E, C, H> {
        LiveVirtualRaindexBuilder::new(orderbook, sync_engine, code_cache, interpreter_host)
    }

    pub fn status(&self) -> LiveStatus {
        self.inner.read().status.clone()
    }

    pub async fn sync_once(&self) -> Result<LiveStatus> {
        let cursor = self.inner.read().cursor.clone();
        let poll = self.sync_engine.poll(self.orderbook, cursor).await?;

        let mut progress = SyncProgress::default();
        let mut cursor_to_persist = None;
        let mut snapshot_to_persist = None;
        let status_to_report;

        let mut pending_union: Vec<ArtifactId> = Vec::new();
        let mut mutation_batches = poll.mutation_batches;
        let bytecode = poll.bytecode;
        let pending_artifacts = poll.pending_artifacts;
        let next_cursor = poll.next_cursor;
        let _heartbeat = poll.heartbeat;

        {
            let mut guard = self.inner.write();
            progress.cached_artifacts = bytecode.len();

            for artifact in &bytecode {
                if let Err(err) = self.code_cache.ingest(artifact) {
                    guard.status = LiveStatus::errored(err.to_string());
                    return Err(err);
                }
            }

            for envelope in mutation_batches.drain(..) {
                if envelope.is_empty() {
                    continue;
                }
                let required = collect_required_artifacts(&envelope);
                if required
                    .iter()
                    .all(|artifact| self.code_cache.is_ready(artifact))
                {
                    if let Err(err) = apply_envelope(&mut guard.raindex, &envelope) {
                        guard.status = LiveStatus::errored(err.to_string());
                        return Err(err);
                    }
                    progress.applied_batches += 1;
                    progress.mutation_count += envelope.mutations.len();
                } else {
                    guard.warmup_queue.enqueue(envelope, required);
                }
            }

            let ready_queue = guard
                .warmup_queue
                .take_ready(|artifact| self.code_cache.is_ready(artifact));
            for envelope in ready_queue {
                if envelope.is_empty() {
                    continue;
                }
                if let Err(err) = apply_envelope(&mut guard.raindex, &envelope) {
                    guard.status = LiveStatus::errored(err.to_string());
                    return Err(err);
                }
                progress.applied_batches += 1;
                progress.mutation_count += envelope.mutations.len();
            }

            progress.deferred_mutations = guard.warmup_queue.len();

            if let Some(cursor) = next_cursor.clone() {
                guard.cursor = Some(cursor.clone());
                cursor_to_persist = Some(cursor);
            }

            if progress.applied_batches > 0 {
                snapshot_to_persist = Some(SnapshotBundle::from_snapshot(
                    self.orderbook,
                    guard.raindex.snapshot(),
                ));
            }

            let mut pending: HashSet<ArtifactId> =
                guard.warmup_queue.pending_artifacts().into_iter().collect();
            for artifact in pending_artifacts {
                pending.insert(artifact);
            }

            pending_union.extend(pending.iter().copied());

            guard.status = if !pending_union.is_empty() {
                LiveStatus::with_pending(pending_union.clone())
            } else if progress.applied_batches > 0 || progress.cached_artifacts > 0 {
                LiveStatus::syncing_ready()
            } else {
                LiveStatus::idle()
            };

            status_to_report = guard.status.clone();
        }

        if let Some(cursor) = cursor_to_persist {
            self.cursor_store
                .persist_cursor(self.orderbook, cursor)
                .await?;
        }

        if let Some(bundle) = snapshot_to_persist {
            self.snapshot_store.persist_snapshot(bundle).await?;
        }

        if let Some(metrics) = &self.metrics {
            metrics.record_progress(&progress).await;
            metrics.record_status(&status_to_report).await;
        }

        Ok(status_to_report)
    }
}

fn apply_envelope<C, H>(
    raindex: &mut VirtualRaindex<C, H>,
    envelope: &MutationEnvelope,
) -> Result<()>
where
    C: LiveCodeCache,
    H: InterpreterHost,
{
    if envelope.is_empty() {
        return Ok(());
    }
    raindex.apply_mutations(&envelope.mutations)
}

fn collect_required_artifacts(envelope: &MutationEnvelope) -> Vec<ArtifactId> {
    let mut required = HashSet::new();
    for mutation in &envelope.mutations {
        collect_from_mutation(mutation, &mut required);
    }
    required.into_iter().collect()
}

fn collect_from_mutation(mutation: &RaindexMutation, required: &mut HashSet<ArtifactId>) {
    match mutation {
        RaindexMutation::SetOrders { orders } => {
            for order in orders {
                required.insert(ArtifactId::new(
                    order.evaluable.interpreter,
                    BytecodeKind::Interpreter,
                ));
                required.insert(ArtifactId::new(order.evaluable.store, BytecodeKind::Store));
            }
        }
        RaindexMutation::Batch(batch) => {
            for inner in batch {
                collect_from_mutation(inner, required);
            }
        }
        _ => {}
    }
}
