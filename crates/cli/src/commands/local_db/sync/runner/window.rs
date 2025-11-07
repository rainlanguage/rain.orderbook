use std::str::FromStr;

use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use rain_orderbook_common::local_db::OrderbookIdentifier;

use super::super::data_source::SyncDataSource;
use super::super::storage::fetch_last_synced;
use super::SyncParams;

pub(super) struct SyncWindow {
    pub(super) last_synced_block: u64,
    pub(super) start_block: u64,
    pub(super) target_block: u64,
    pub(super) latest_block: u64,
    pub(super) start_adjustment: Option<StartAdjustment>,
    pub(super) end_clamp: Option<EndClamp>,
    pub(super) noop: bool,
}

pub(super) struct StartAdjustment {
    previous: u64,
    new_start: u64,
    reason: StartAdjustmentReason,
}

pub(super) enum StartAdjustmentReason {
    BeforeDeployment { deployment_block: u64 },
    BehindLastSynced,
}

pub(super) struct EndClamp {
    requested: u64,
    latest_block: u64,
}

pub(super) async fn compute_sync_window<D>(
    db_path: &str,
    data_source: &D,
    params: &SyncParams<'_>,
) -> Result<SyncWindow>
where
    D: SyncDataSource + Send + Sync,
{
    let last_synced_block = fetch_last_synced(
        db_path,
        &OrderbookIdentifier::new(
            params.chain_id,
            Address::from_str(params.orderbook_address)?,
        ),
    )
    .await?;

    let mut start_block = params
        .start_block
        .unwrap_or_else(|| default_start_block(last_synced_block, params.deployment_block));
    let mut start_adjustment = None;

    if last_synced_block > 0 && start_block <= last_synced_block {
        let new_start = last_synced_block.checked_add(1).ok_or_else(|| {
            anyhow!(
                "last synced block {} overflowed when incrementing",
                last_synced_block
            )
        })?;
        start_adjustment = Some(StartAdjustment {
            previous: start_block,
            new_start,
            reason: StartAdjustmentReason::BehindLastSynced,
        });
        start_block = new_start;
    }

    if last_synced_block == 0 && start_block < params.deployment_block {
        start_adjustment = Some(StartAdjustment {
            previous: start_block,
            new_start: params.deployment_block,
            reason: StartAdjustmentReason::BeforeDeployment {
                deployment_block: params.deployment_block,
            },
        });
        start_block = params.deployment_block;
    }

    let latest_block = data_source.latest_block().await?;
    let mut target_block = params.end_block.unwrap_or(latest_block);
    let mut end_clamp = None;
    if target_block > latest_block {
        end_clamp = Some(EndClamp {
            requested: target_block,
            latest_block,
        });
        target_block = latest_block;
    }

    let noop = start_block > target_block;

    Ok(SyncWindow {
        last_synced_block,
        start_block,
        target_block,
        latest_block,
        start_adjustment,
        end_clamp,
        noop,
    })
}

pub(super) fn default_start_block(last_synced_block: u64, deployment_block: u64) -> u64 {
    if last_synced_block == 0 {
        deployment_block
    } else {
        last_synced_block + 1
    }
}

impl StartAdjustment {
    pub(super) fn message(&self, last_synced_block: u64) -> String {
        match self.reason {
            StartAdjustmentReason::BehindLastSynced => format!(
                "Provided start block {} is <= last_synced_block {}; bumping to {}",
                self.previous, last_synced_block, self.new_start
            ),
            StartAdjustmentReason::BeforeDeployment { deployment_block } => format!(
                "Start block {} is before deployment block {}; using deployment block",
                self.previous, deployment_block
            ),
        }
    }
}

impl EndClamp {
    pub(super) fn message(&self) -> String {
        format!(
            "Requested end block {} is beyond chain head {}; clamping",
            self.requested, self.latest_block
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use async_trait::async_trait;
    use rain_orderbook_common::local_db::decode::{DecodedEvent, DecodedEventData};
    use rain_orderbook_common::rpc_client::LogEntryResponse;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::executor::RusqliteExecutor;
    use crate::commands::local_db::sync::storage::DEFAULT_SCHEMA_SQL;
    use rain_orderbook_common::local_db::query::{
        update_last_synced_block::build_update_last_synced_block_stmt, LocalDbQueryExecutor,
        SqlStatement, SqlStatementBatch,
    };

    struct MockDataSource {
        latest_block: u64,
        rpc_urls: Vec<Url>,
    }

    #[async_trait]
    impl SyncDataSource for MockDataSource {
        async fn latest_block(&self) -> Result<u64> {
            Ok(self.latest_block)
        }

        async fn fetch_events(
            &self,
            _orderbook_address: &str,
            _start_block: u64,
            _end_block: u64,
        ) -> Result<Vec<LogEntryResponse>> {
            Ok(vec![])
        }

        async fn fetch_store_set_events(
            &self,
            _store_addresses: &[String],
            _start_block: u64,
            _end_block: u64,
        ) -> Result<Vec<LogEntryResponse>> {
            Ok(vec![])
        }

        fn decode_events(
            &self,
            _events: &[LogEntryResponse],
        ) -> Result<Vec<DecodedEventData<DecodedEvent>>> {
            Ok(vec![])
        }

        fn events_to_sql(
            &self,
            _ob_id: &OrderbookIdentifier,
            _decoded_events: &[DecodedEventData<DecodedEvent>],
            _decimals_by_token: &HashMap<Address, u8>,
        ) -> Result<SqlStatementBatch> {
            Ok(SqlStatementBatch::new())
        }

        fn raw_events_to_statements(
            &self,
            _ob_id: &OrderbookIdentifier,
            _: &[LogEntryResponse],
        ) -> Result<SqlStatementBatch> {
            Ok(SqlStatementBatch::new())
        }

        fn rpc_urls(&self) -> &[Url] {
            &self.rpc_urls
        }
    }

    fn params() -> SyncParams<'static> {
        SyncParams {
            chain_id: 1,
            orderbook_address: "0x1111111111111111111111111111111111111111",
            deployment_block: 50,
            start_block: None,
            end_block: None,
        }
    }

    #[tokio::test]
    async fn default_start_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("window.db");
        let db_path_str = db_path.to_string_lossy();
        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();

        let data_source = MockDataSource {
            latest_block: 100,
            rpc_urls: vec![Url::parse("http://rpc").unwrap()],
        };

        let window = compute_sync_window(&db_path_str, &data_source, &params())
            .await
            .unwrap();

        assert_eq!(window.start_block, 50);
        assert_eq!(window.target_block, 100);
        assert!(window.start_adjustment.is_none());
        assert!(window.end_clamp.is_none());
    }

    #[tokio::test]
    async fn adjusts_when_start_before_last_synced() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("window.db");
        let db_path_str = db_path.to_string_lossy();
        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();
        exec.query_text(&build_update_last_synced_block_stmt(
            &OrderbookIdentifier::new(1, Address::from([0x11; 20])),
            80,
        ))
        .await
        .unwrap();

        let data_source = MockDataSource {
            latest_block: 120,
            rpc_urls: vec![Url::parse("http://rpc").unwrap()],
        };

        let mut params = params();
        params.start_block = Some(75);

        let window = compute_sync_window(&db_path_str, &data_source, &params)
            .await
            .unwrap();

        assert_eq!(window.start_block, 81);
        assert!(window.start_adjustment.is_some());
    }

    #[tokio::test]
    async fn clamps_end_block() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("window.db");
        let db_path_str = db_path.to_string_lossy();
        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();

        let data_source = MockDataSource {
            latest_block: 90,
            rpc_urls: vec![Url::parse("http://rpc").unwrap()],
        };

        let mut params = params();
        params.end_block = Some(100);

        let window = compute_sync_window(&db_path_str, &data_source, &params)
            .await
            .unwrap();

        assert_eq!(window.target_block, 90);
        assert!(window.end_clamp.is_some());
    }

    #[tokio::test]
    async fn noop_when_start_exceeds_target() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("window.db");
        let db_path_str = db_path.to_string_lossy();
        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();

        let data_source = MockDataSource {
            latest_block: 60,
            rpc_urls: vec![Url::parse("http://rpc").unwrap()],
        };

        let mut params = params();
        params.start_block = Some(70);

        let window = compute_sync_window(&db_path_str, &data_source, &params)
            .await
            .unwrap();

        assert!(window.noop);
    }

    #[test]
    fn default_start_block_behaviour() {
        assert_eq!(default_start_block(0, 42), 42);
        assert_eq!(default_start_block(99, 10), 100);
    }
}
