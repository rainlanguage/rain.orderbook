use crate::local_db::pipeline::traits::BootstrapConfig;
use crate::local_db::pipeline::traits::BootstrapPipeline;
use crate::local_db::pipeline::traits::BootstrapState;
use crate::local_db::pipeline::traits::TargetKey;
use crate::local_db::query::clear_tables::clear_tables_stmt;
use crate::local_db::query::create_tables::create_tables_stmt;
use crate::local_db::query::create_tables::REQUIRED_TABLES;
use crate::local_db::query::fetch_db_metadata::{fetch_db_metadata_stmt, DbMetadataRow};
use crate::local_db::query::fetch_tables::{fetch_tables_stmt, TableResponse};
use crate::local_db::query::fetch_target_watermark::fetch_target_watermark_stmt;
use crate::local_db::query::fetch_target_watermark::TargetWatermarkRow;
use crate::local_db::query::insert_db_metadata::insert_db_metadata_stmt;
use crate::local_db::query::LocalDbQueryExecutor;
use crate::local_db::LocalDbError;
use crate::local_db::DATABASE_SCHEMA_VERSION;
use std::collections::HashSet;

/// Default adapter that exposes the shared bootstrap helpers via the
/// BootstrapPipeline trait. The `run` method performs a minimal safe
/// orchestration (ensure schema only). Environment runners can provide
/// their own adapter overriding `run` while reusing these helpers.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultBootstrapAdapter;

impl DefaultBootstrapAdapter {
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait(?Send)]
impl BootstrapPipeline for DefaultBootstrapAdapter {
    async fn ensure_schema<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let rows = db
            .query_json::<Vec<DbMetadataRow>>(&fetch_db_metadata_stmt())
            .await?;
        if let Some(row) = rows.first() {
            let expected = db_schema_version.unwrap_or(DATABASE_SCHEMA_VERSION);
            if row.db_schema_version != expected {
                return Err(LocalDbError::SchemaVersionMismatch {
                    expected,
                    found: row.db_schema_version,
                });
            }
            return Ok(());
        }
        return Err(LocalDbError::MissingDbMetadataRow);
    }

    async fn inspect_state<DB>(
        &self,
        db: &DB,
        target_key: &TargetKey,
    ) -> Result<BootstrapState, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let existing: Vec<TableResponse> = db.query_json(&fetch_tables_stmt()).await?;
        let existing_set: HashSet<String> = existing
            .into_iter()
            .map(|t| t.name.to_ascii_lowercase())
            .collect();

        let has_required_tables = REQUIRED_TABLES
            .iter()
            .all(|&t| existing_set.contains(&t.to_ascii_lowercase()));

        let last_synced_block = if existing_set.contains("target_watermarks") {
            let rows: Vec<TargetWatermarkRow> = db
                .query_json(&fetch_target_watermark_stmt(
                    target_key.chain_id,
                    target_key.orderbook_address,
                ))
                .await?;
            rows.first().map(|r| r.last_block)
        } else {
            None
        };

        Ok(BootstrapState {
            has_required_tables,
            last_synced_block,
        })
    }

    async fn reset_db<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        db.query_text(&clear_tables_stmt()).await?;
        db.query_text(&create_tables_stmt()).await?;
        db.query_text(&insert_db_metadata_stmt(
            db_schema_version.unwrap_or(DATABASE_SCHEMA_VERSION),
        ))
        .await?;
        Ok(())
    }

    async fn run<DB>(&self, _: &DB, _: Option<u32>, _: &BootstrapConfig) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        Err(LocalDbError::MissingBootstrapImplementation)
    }
}
