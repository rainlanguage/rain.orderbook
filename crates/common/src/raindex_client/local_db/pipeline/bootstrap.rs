use crate::local_db::{
    pipeline::{
        adapters::bootstrap::DefaultBootstrapAdapter,
        traits::{BootstrapConfig, BootstrapPipeline, BootstrapState, TargetKey},
    },
    query::{
        fetch_target_watermark::{fetch_target_watermark_stmt, TargetWatermarkRow},
        LocalDbQueryExecutor,
    },
    LocalDbError,
};

const BLOCK_NUMBER_THRESHOLD: u64 = 10_000;

#[derive(Debug, Default, Clone, Copy)]
pub struct ClientBootstrapAdapter;

impl ClientBootstrapAdapter {
    fn check_threshold(
        &self,
        latest_block: u64,
        last_synced_block: Option<u64>,
    ) -> Result<(), LocalDbError> {
        if let Some(last_block) = last_synced_block {
            if latest_block.saturating_sub(last_block) > BLOCK_NUMBER_THRESHOLD {
                return Err(LocalDbError::BlockSyncThresholdExceeded {
                    latest_block,
                    last_indexed_block: last_block,
                    threshold: BLOCK_NUMBER_THRESHOLD,
                });
            }
        }

        Ok(())
    }

    async fn is_fresh_db<E: LocalDbQueryExecutor + ?Sized>(
        self,
        db: &E,
        target_key: &TargetKey,
    ) -> Result<bool, LocalDbError> {
        let rows: Vec<TargetWatermarkRow> = db
            .query_json(&fetch_target_watermark_stmt(
                target_key.chain_id,
                target_key.orderbook_address,
            ))
            .await?;
        Ok(rows.is_empty())
    }
}

#[async_trait::async_trait(?Send)]
impl BootstrapPipeline for ClientBootstrapAdapter {
    async fn ensure_schema<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        DefaultBootstrapAdapter::new()
            .ensure_schema(db, db_schema_version)
            .await
    }

    async fn inspect_state<DB>(
        &self,
        db: &DB,
        target_key: &TargetKey,
    ) -> Result<BootstrapState, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        DefaultBootstrapAdapter::new()
            .inspect_state(db, target_key)
            .await
    }

    async fn reset_db<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        DefaultBootstrapAdapter::new()
            .reset_db(db, db_schema_version)
            .await
    }

    async fn run<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
        config: &BootstrapConfig,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let BootstrapState {
            has_required_tables,
            last_synced_block,
        } = self.inspect_state(db, &config.target_key).await?;

        if !has_required_tables {
            self.reset_db(db, db_schema_version).await?;
        }

        match self.ensure_schema(db, db_schema_version).await {
            Ok(_) => {}
            Err(LocalDbError::MissingDbMetadataRow)
            | Err(LocalDbError::SchemaVersionMismatch { .. }) => {
                self.reset_db(db, db_schema_version).await?;
            }
            Err(err) => return Err(err),
        }

        if let Some(dump_stmt) = config.dump_stmt.as_ref() {
            if self.is_fresh_db(db, &config.target_key).await? {
                db.query_text(dump_stmt).await?;
                return Ok(());
            }

            match self.check_threshold(config.latest_block, last_synced_block) {
                Ok(_) => {}
                Err(_) => {
                    self.reset_db(db, db_schema_version).await?;
                    db.query_text(dump_stmt).await?;
                }
            }
        }

        Ok(())
    }
}
