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
    pub fn new() -> Self {
        Self {}
    }

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use crate::local_db::pipeline::traits::BootstrapConfig;
    use crate::local_db::query::clear_tables::clear_tables_stmt;
    use crate::local_db::query::create_tables::create_tables_stmt;
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use crate::local_db::query::fetch_db_metadata::{fetch_db_metadata_stmt, DbMetadataRow};
    use crate::local_db::query::fetch_tables::{fetch_tables_stmt, TableResponse};
    use crate::local_db::query::fetch_target_watermark::{
        fetch_target_watermark_stmt, TargetWatermarkRow,
    };
    use crate::local_db::query::insert_db_metadata::insert_db_metadata_stmt;
    use crate::local_db::query::FromDbJson;
    use crate::local_db::query::{
        LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    };
    use crate::local_db::DATABASE_SCHEMA_VERSION;
    use alloy::primitives::Address;
    use async_trait::async_trait;
    use serde_json::json;

    #[derive(Default)]
    struct MockDb {
        json_map: HashMap<String, String>,
        text_map: HashMap<String, String>,
        calls_text: Mutex<Vec<String>>,
    }

    impl MockDb {
        fn with_json(mut self, stmt: &SqlStatement, value: serde_json::Value) -> Self {
            self.json_map
                .insert(stmt.sql().to_string(), value.to_string());
            self
        }
        fn with_text(mut self, stmt: &SqlStatement, value: &str) -> Self {
            self.text_map
                .insert(stmt.sql().to_string(), value.to_string());
            self
        }
        fn calls(&self) -> Vec<String> {
            self.calls_text.lock().unwrap().clone()
        }
    }

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for MockDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            for stmt in batch {
                let _ = self.query_text(stmt).await?;
            }
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            let sql = stmt.sql();
            let Some(body) = self.json_map.get(sql) else {
                return Err(LocalDbQueryError::database("no json for sql"));
            };
            serde_json::from_str::<T>(body)
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            let sql = stmt.sql();
            self.calls_text.lock().unwrap().push(sql.to_string());
            let Some(body) = self.text_map.get(sql) else {
                return Err(LocalDbQueryError::database("no text for sql"));
            };
            Ok(body.clone())
        }
    }

    fn target_key() -> TargetKey {
        TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        }
    }

    fn cfg_with_dump(latest_block: u64) -> BootstrapConfig {
        BootstrapConfig {
            target_key: target_key(),
            dump_stmt: Some(SqlStatement::new("--dump-sql")),
            latest_block,
        }
    }

    #[tokio::test]
    async fn run_resets_when_missing_tables_and_no_dump() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = json!([]); // no required tables
        let db_meta_row = DbMetadataRow {
            id: 1,
            db_schema_version: DATABASE_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([db_meta_row]))
            // reset_db calls
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok");

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: None,
            latest_block: 0,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        // Expect reset sequence only
        assert_eq!(calls[0], clear_tables_stmt().sql().to_string());
        assert_eq!(calls[1], create_tables_stmt().sql().to_string());
        assert_eq!(
            calls[2],
            insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION)
                .sql()
                .to_string()
        );
    }

    #[tokio::test]
    async fn run_resets_on_missing_db_metadata_row() {
        let adapter = ClientBootstrapAdapter::new();
        // All required tables present
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([])) // triggers reset
            // inspect_state will look for watermark since table exists
            .with_json(&fetch_target_watermark_stmt(1, Address::ZERO), json!([]))
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok");

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: None,
            latest_block: 0,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        assert!(calls.contains(&clear_tables_stmt().sql().to_string()));
        assert!(calls.contains(&create_tables_stmt().sql().to_string()));
        assert!(calls.contains(
            &insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION)
                .sql()
                .to_string()
        ));
    }

    #[tokio::test]
    async fn run_resets_on_schema_mismatch() {
        let adapter = ClientBootstrapAdapter::new();
        // All required tables present
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DATABASE_SCHEMA_VERSION + 1,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([db_row])) // mismatch triggers reset
            // inspect_state will look for watermark since table exists
            .with_json(&fetch_target_watermark_stmt(1, Address::ZERO), json!([]))
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok");

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: None,
            latest_block: 0,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        assert!(calls.contains(&clear_tables_stmt().sql().to_string()));
        assert!(calls.contains(&create_tables_stmt().sql().to_string()));
    }

    #[tokio::test]
    async fn run_applies_dump_on_fresh_db() {
        let adapter = ClientBootstrapAdapter::new();
        // Start without tables to force reset, then fresh watermark after reset
        let tables_json = json!([]);
        let db_meta_row = DbMetadataRow {
            id: 1,
            db_schema_version: DATABASE_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };

        let dump_stmt = SqlStatement::new("--dump-sql");

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([db_meta_row]))
            // fresh DB check: no rows
            .with_json(&fetch_target_watermark_stmt(1, Address::ZERO), json!([]))
            // reset + dump
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok")
            .with_text(&dump_stmt, "ok");

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: 100,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        assert!(calls.contains(&dump_stmt.sql().to_string()));
    }

    #[tokio::test]
    async fn run_skips_dump_when_within_threshold() {
        let adapter = ClientBootstrapAdapter::new();
        // Tables present, watermark exists => not fresh
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let last_synced = 90_000u64;
        let latest = last_synced + 9_000; // below 10_000 threshold
        let watermark_row = TargetWatermarkRow {
            chain_id: 1,
            orderbook_address: Address::ZERO,
            last_block: last_synced,
            last_hash: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(1, Address::ZERO),
                json!([watermark_row.clone()]),
            )
            // ensure_schema ok
            .with_json(
                &fetch_db_metadata_stmt(),
                json!([DbMetadataRow {
                    id: 1,
                    db_schema_version: DATABASE_SCHEMA_VERSION,
                    created_at: None,
                    updated_at: None
                }]),
            );

        let cfg = cfg_with_dump(latest);

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        // No reset and no dump calls expected
        assert!(calls.is_empty());
    }

    #[tokio::test]
    async fn run_resets_and_applies_dump_when_threshold_exceeded() {
        let adapter = ClientBootstrapAdapter::new();
        // Tables present, watermark exists => not fresh
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let last_synced = 50_000u64;
        let latest = last_synced + 20_001; // above threshold
        let watermark_row = TargetWatermarkRow {
            chain_id: 1,
            orderbook_address: Address::ZERO,
            last_block: last_synced,
            last_hash: None,
            updated_at: None,
        };
        let dump_stmt = SqlStatement::new("--dump-sql");

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(1, Address::ZERO),
                json!([watermark_row]),
            )
            // ensure_schema ok
            .with_json(
                &fetch_db_metadata_stmt(),
                json!([DbMetadataRow {
                    id: 1,
                    db_schema_version: DATABASE_SCHEMA_VERSION,
                    created_at: None,
                    updated_at: None
                }]),
            )
            // reset + dump
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok")
            .with_text(&dump_stmt, "ok");

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: latest,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        assert!(calls.contains(&clear_tables_stmt().sql().to_string()));
        assert!(calls.contains(&dump_stmt.sql().to_string()));
    }

    #[tokio::test]
    async fn run_skips_dump_on_threshold_boundary() {
        let adapter = ClientBootstrapAdapter::new();
        // Tables present, watermark exists => not fresh
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let last_synced = 100_000u64;
        let latest = last_synced + BLOCK_NUMBER_THRESHOLD; // exactly at threshold
        let watermark_row = TargetWatermarkRow {
            chain_id: 1,
            orderbook_address: Address::ZERO,
            last_block: last_synced,
            last_hash: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(1, Address::ZERO),
                json!([watermark_row]),
            )
            // ensure_schema ok
            .with_json(
                &fetch_db_metadata_stmt(),
                json!([DbMetadataRow {
                    id: 1,
                    db_schema_version: DATABASE_SCHEMA_VERSION,
                    created_at: None,
                    updated_at: None
                }]),
            );

        let cfg = cfg_with_dump(latest);

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        // No reset and no dump calls expected
        assert!(calls.is_empty());
    }

    #[tokio::test]
    async fn run_does_nothing_when_dump_absent_even_if_threshold_exceeded() {
        let adapter = ClientBootstrapAdapter::new();
        // Tables present, watermark exists => not fresh
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let last_synced = 200_000u64;
        let latest = last_synced + BLOCK_NUMBER_THRESHOLD + 1; // exceed threshold
        let watermark_row = TargetWatermarkRow {
            chain_id: 1,
            orderbook_address: Address::ZERO,
            last_block: last_synced,
            last_hash: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(1, Address::ZERO),
                json!([watermark_row]),
            )
            // ensure_schema ok
            .with_json(
                &fetch_db_metadata_stmt(),
                json!([DbMetadataRow {
                    id: 1,
                    db_schema_version: DATABASE_SCHEMA_VERSION,
                    created_at: None,
                    updated_at: None
                }]),
            );

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: None,
            latest_block: latest,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        // With no dump configured, exceeding threshold should not trigger reset/dump
        assert!(calls.is_empty());
    }

    #[tokio::test]
    async fn run_propagates_unexpected_ensure_schema_error() {
        let adapter = ClientBootstrapAdapter::new();
        // Make inspect_state succeed, but ensure_schema fails with a query error
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let watermark_row = TargetWatermarkRow {
            chain_id: 1,
            orderbook_address: Address::ZERO,
            last_block: 1,
            last_hash: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(1, Address::ZERO),
                json!([watermark_row]),
            ); // intentionally omit fetch_db_metadata to force ensure_schema error

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: Some(SqlStatement::new("--dump-sql")),
            latest_block: 2,
        };

        let err = adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap_err();

        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }

        // Should not have attempted any reset/dump text queries
        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn run_resets_and_applies_dump_when_ensure_schema_missing_metadata() {
        let adapter = ClientBootstrapAdapter::new();
        // All required tables present so inspect_state succeeds; ensure_schema sees no metadata row
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let dump_stmt = SqlStatement::new("--dump-sql");

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            // inspect_state watermark read (empty -> fresh after reset)
            .with_json(&fetch_target_watermark_stmt(1, Address::ZERO), json!([]))
            // ensure_schema -> missing metadata row
            .with_json(&fetch_db_metadata_stmt(), json!([]))
            // reset + dump
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok")
            .with_text(&dump_stmt, "ok");

        let cfg = BootstrapConfig {
            target_key: target_key(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: 1,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        assert!(calls.contains(&clear_tables_stmt().sql().to_string()));
        assert!(calls.contains(&dump_stmt.sql().to_string()));
    }
}
