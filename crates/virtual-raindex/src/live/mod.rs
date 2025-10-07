//! Live synchronization primitives for the Virtual Raindex engine.

mod adapter;
mod bytecode;
mod cache_ext;
mod facade;
mod status;
mod sync_engine;

pub mod stub;

#[cfg(test)]
mod tests;

pub use adapter::{
    CursorStore, InMemoryCursorStore, InMemorySnapshotStore, MetricsSink, NoopMetrics,
    SnapshotStore,
};
pub use cache_ext::LiveCodeCache;
pub use facade::{LiveVirtualRaindex, LiveVirtualRaindexBuilder};
pub use status::{LiveAdvisory, LivePhase, LiveStatus, PendingArtifacts, SyncProgress};
pub use sync_engine::{
    ArtifactBatch, ArtifactId, BytecodeArtifact, MutationEnvelope, SyncEngine, SyncPoll,
};
