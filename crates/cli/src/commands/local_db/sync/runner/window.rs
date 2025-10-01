use anyhow::Result;

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
    let last_synced_block = fetch_last_synced(db_path)?;

    let mut start_block = params
        .start_block
        .unwrap_or_else(|| default_start_block(last_synced_block, params.deployment_block));
    let mut start_adjustment = None;

    if last_synced_block > 0 && start_block <= last_synced_block {
        let new_start = last_synced_block + 1;
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
    use crate::commands::local_db::sqlite::sqlite_execute;
    use crate::commands::local_db::sync::storage::DEFAULT_SCHEMA_SQL;
    use async_trait::async_trait;
    use serde_json::json;
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::sync::data_source::SyncDataSource;

    struct MockDataSource {
        latest_block: u64,
    }

    #[async_trait]
    impl SyncDataSource for MockDataSource {
        async fn latest_block(&self) -> Result<u64> {
            Ok(self.latest_block)
        }

        async fn fetch_events(&self, _: &str, _: u64, _: u64) -> Result<serde_json::Value> {
            Ok(json!([]))
        }

        async fn fetch_store_set_events(
            &self,
            _: &[String],
            _: u64,
            _: u64,
        ) -> Result<serde_json::Value> {
            Ok(json!([]))
        }

        fn decode_events(&self, events: serde_json::Value) -> Result<serde_json::Value> {
            Ok(events)
        }

        fn events_to_sql(&self, _: serde_json::Value, _: u64, _: &str) -> Result<String> {
            Ok(String::new())
        }

        fn rpc_urls(&self) -> &[Url] {
            &[]
        }
    }

    #[tokio::test]
    async fn default_start_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("window.db");
        let db_path_str = db_path.to_string_lossy();
        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();

        let data_source = MockDataSource { latest_block: 10 };
        let params = SyncParams {
            chain_id: 1,
            orderbook_address: "0xfeed",
            deployment_block: 5,
            start_block: None,
            end_block: None,
        };

        let window = compute_sync_window(&db_path_str, &data_source, &params)
            .await
            .unwrap();
        assert_eq!(window.start_block, 5);
        assert_eq!(window.last_synced_block, 0);
    }

    #[test]
    fn default_start_block_matches_previous_behavior() {
        assert_eq!(default_start_block(0, 123), 123);
        assert_eq!(default_start_block(10, 5), 11);
    }
}
