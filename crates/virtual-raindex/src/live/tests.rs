#[cfg(test)]
mod live_tests {
    use std::sync::Arc;

    use alloy::primitives::Address;
    use async_trait::async_trait;
    use parking_lot::RwLock;

    use super::super::stub::{StubCursor, StubSyncEngine};
    use super::super::{
        InMemoryCursorStore, InMemorySnapshotStore, LiveAdvisory, LivePhase, LiveStatus,
        LiveVirtualRaindex, MetricsSink, SyncProgress,
    };
    use crate::live::adapter::{CursorStore, SnapshotStore};
    use crate::{cache::StaticCodeCache, host::RevmInterpreterHost, Result};

    #[derive(Default)]
    struct RecordingMetrics {
        statuses: RwLock<Vec<LivePhase>>,
        applied: RwLock<Vec<usize>>,
    }

    #[async_trait]
    impl MetricsSink for RecordingMetrics {
        async fn record_status(&self, status: &LiveStatus) {
            self.statuses.write().push(status.phase);
        }

        async fn record_progress(&self, progress: &SyncProgress) {
            self.applied.write().push(progress.applied_batches);
        }
    }

    #[tokio::test]
    async fn demo_script_progression() -> Result<()> {
        let orderbook = Address::from([0x11u8; 20]);
        let sync_engine = Arc::new(StubSyncEngine::demo());
        let code_cache = Arc::new(StaticCodeCache::default());
        let host = Arc::new(RevmInterpreterHost::new(code_cache.clone()));
        let snapshot_store = Arc::new(InMemorySnapshotStore::new());
        let cursor_store = Arc::new(InMemoryCursorStore::<StubCursor>::new());
        let metrics = Arc::new(RecordingMetrics::default());

        let live = LiveVirtualRaindex::builder(
            orderbook,
            sync_engine.clone(),
            code_cache.clone(),
            host.clone(),
        )
        .with_snapshot_store(snapshot_store.clone())
        .with_cursor_store(cursor_store.clone())
        .with_metrics(metrics.clone())
        .build()
        .await?;

        assert_eq!(live.status().phase, LivePhase::Idle);

        let status1 = live.sync_once().await?;
        assert_eq!(status1.phase, LivePhase::Syncing);
        assert!(matches!(
            status1.advisories.as_slice(),
            [LiveAdvisory::Ready]
        ));

        let cursor1 = cursor_store.load_cursor(orderbook).await?;
        assert_eq!(cursor1, Some(StubCursor(1)));

        let snapshot1 = snapshot_store.load_snapshot(orderbook).await?.unwrap();
        assert_eq!(snapshot1.env.block_number, 1);
        assert_eq!(snapshot1.env.timestamp, 1_700_000_000);

        let status2 = live.sync_once().await?;
        assert_eq!(status2.phase, LivePhase::PendingArtifacts);
        assert_eq!(status2.pending.artifacts.len(), 2);

        let status3 = live.sync_once().await?;
        assert_eq!(status3.phase, LivePhase::Syncing);
        assert!(status3.pending.artifacts.is_empty());

        let cursor3 = cursor_store.load_cursor(orderbook).await?;
        assert_eq!(cursor3, Some(StubCursor(3)));

        let snapshot3 = snapshot_store.load_snapshot(orderbook).await?.unwrap();
        assert_eq!(snapshot3.orders.len(), 1);

        let status4 = live.sync_once().await?;
        assert_eq!(status4.phase, LivePhase::Idle);

        assert_eq!(metrics.statuses.read().len(), 4);
        let applied = metrics.applied.read();
        assert_eq!(applied.as_slice(), &[1, 0, 1, 0]);

        Ok(())
    }

    #[test]
    fn demo_fixture_parses() {
        let engine = StubSyncEngine::demo();
        assert_eq!(engine.remaining(), 4);
    }
}
