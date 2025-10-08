use std::collections::HashMap;
use std::fmt::Debug;

use alloy::primitives::Address;
use async_trait::async_trait;
use parking_lot::RwLock;

use crate::snapshot::SnapshotBundle;

use super::status::{LiveStatus, SyncProgress};

#[async_trait]
pub trait SnapshotStore: Send + Sync {
    async fn load_snapshot(&self, orderbook: Address) -> crate::Result<Option<SnapshotBundle>>;
    async fn persist_snapshot(&self, bundle: SnapshotBundle) -> crate::Result<()>;
}

#[async_trait]
pub trait CursorStore<C>: Send + Sync
where
    C: Clone + Send + Sync + Debug + Eq + PartialEq + 'static,
{
    async fn load_cursor(&self, orderbook: Address) -> crate::Result<Option<C>>;
    async fn persist_cursor(&self, orderbook: Address, cursor: C) -> crate::Result<()>;
}

#[async_trait]
pub trait MetricsSink: Send + Sync {
    async fn record_status(&self, _status: &LiveStatus) {}
    async fn record_progress(&self, _progress: &SyncProgress) {}
}

pub struct InMemorySnapshotStore {
    snapshots: RwLock<HashMap<Address, SnapshotBundle>>,
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_initial<I>(snapshots: I) -> Self
    where
        I: IntoIterator<Item = SnapshotBundle>,
    {
        let mut map = HashMap::new();
        for snapshot in snapshots {
            map.insert(snapshot.orderbook, snapshot);
        }
        Self {
            snapshots: RwLock::new(map),
        }
    }
}

#[async_trait]
impl SnapshotStore for InMemorySnapshotStore {
    async fn load_snapshot(&self, orderbook: Address) -> crate::Result<Option<SnapshotBundle>> {
        Ok(self.snapshots.read().get(&orderbook).cloned())
    }

    async fn persist_snapshot(&self, bundle: SnapshotBundle) -> crate::Result<()> {
        self.snapshots.write().insert(bundle.orderbook, bundle);
        Ok(())
    }
}

pub struct InMemoryCursorStore<C>
where
    C: Clone + Send + Sync + Debug + Eq + PartialEq + 'static,
{
    cursors: RwLock<HashMap<Address, C>>,
}

impl<C> InMemoryCursorStore<C>
where
    C: Clone + Send + Sync + Debug + Eq + PartialEq + 'static,
{
    pub fn new() -> Self {
        Self {
            cursors: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_initial<I>(initial: I) -> Self
    where
        I: IntoIterator<Item = (Address, C)>,
    {
        Self {
            cursors: RwLock::new(initial.into_iter().collect()),
        }
    }
}

#[async_trait]
impl<C> CursorStore<C> for InMemoryCursorStore<C>
where
    C: Clone + Send + Sync + Debug + Eq + PartialEq + 'static,
{
    async fn load_cursor(&self, orderbook: Address) -> crate::Result<Option<C>> {
        Ok(self.cursors.read().get(&orderbook).cloned())
    }

    async fn persist_cursor(&self, orderbook: Address, cursor: C) -> crate::Result<()> {
        self.cursors.write().insert(orderbook, cursor);
        Ok(())
    }
}

pub struct NoopMetrics;

#[async_trait]
impl MetricsSink for NoopMetrics {}

impl<C> Default for InMemoryCursorStore<C>
where
    C: Clone + Send + Sync + Debug + Eq + PartialEq + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InMemorySnapshotStore {
    fn default() -> Self {
        Self::new()
    }
}
