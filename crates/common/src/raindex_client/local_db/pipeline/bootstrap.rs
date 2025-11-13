use crate::local_db::{
    pipeline::adapters::bootstrap::{BootstrapConfig, BootstrapPipeline, BootstrapState},
    query::{
        fetch_target_watermark::{fetch_target_watermark_stmt, TargetWatermarkRow},
        LocalDbQueryExecutor,
    },
    LocalDbError, OrderbookIdentifier,
};
use alloy::primitives::Address;

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
        block_number_threshold: u32,
    ) -> Result<(), LocalDbError> {
        if let Some(last_block) = last_synced_block {
            let threshold = u64::from(block_number_threshold);
            if latest_block.saturating_sub(last_block) > threshold {
                return Err(LocalDbError::BlockSyncThresholdExceeded {
                    latest_block,
                    last_indexed_block: last_block,
                    threshold,
                });
            }
        }

        Ok(())
    }

    async fn is_fresh_db<E: LocalDbQueryExecutor + ?Sized>(
        self,
        db: &E,
        ob_id: &OrderbookIdentifier,
    ) -> Result<bool, LocalDbError> {
        let rows: Vec<TargetWatermarkRow> =
            db.query_json(&fetch_target_watermark_stmt(ob_id)).await?;
        Ok(rows.is_empty())
    }
}

#[async_trait::async_trait(?Send)]
impl BootstrapPipeline for ClientBootstrapAdapter {
    async fn engine_run<DB>(&self, db: &DB, config: &BootstrapConfig) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let BootstrapState {
            last_synced_block, ..
        } = self.inspect_state(db, &config.ob_id).await?;

        if let Some(dump_stmt) = config.dump_stmt.as_ref() {
            if self.is_fresh_db(db, &config.ob_id).await? {
                db.query_text(dump_stmt).await?;
                return Ok(());
            }

            match self.check_threshold(
                config.latest_block,
                last_synced_block,
                config.block_number_threshold,
            ) {
                Ok(_) => {}
                Err(_) => {
                    self.clear_orderbook_data(db, &config.ob_id).await?;
                    db.query_text(dump_stmt).await?;
                }
            }
        }

        Ok(())
    }

    async fn runner_run<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let BootstrapState {
            has_required_tables,
            ..
        } = self
            .inspect_state(db, &OrderbookIdentifier::new(0, Address::ZERO))
            .await?;

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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use crate::local_db::query::clear_orderbook_data::clear_orderbook_data_batch;
    use crate::local_db::query::clear_tables::clear_tables_stmt;
    use crate::local_db::query::create_tables::create_tables_stmt;
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use crate::local_db::query::create_views::create_views_batch;
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
    use alloy::primitives::{Address, Bytes};
    use async_trait::async_trait;
    use rain_orderbook_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
    use serde_json::json;
    use std::str::FromStr;

    const TEST_BLOCK_NUMBER_THRESHOLD: u32 = 10_000;

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

    fn with_view_creation_sql(db: MockDb) -> MockDb {
        create_views_batch()
            .statements()
            .iter()
            .fold(db, |db_acc, stmt| db_acc.with_text(stmt, "ok"))
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

    fn sample_ob_id() -> OrderbookIdentifier {
        OrderbookIdentifier {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        }
    }

    fn runner_ob_id() -> OrderbookIdentifier {
        OrderbookIdentifier::new(0, Address::ZERO)
    }

    fn cfg_with_dump(latest_block: u64) -> BootstrapConfig {
        BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(SqlStatement::new("--dump-sql")),
            latest_block,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        }
    }

    fn required_tables_json() -> serde_json::Value {
        serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn watermark_row(last_block: u64) -> TargetWatermarkRow {
        TargetWatermarkRow {
            chain_id: sample_ob_id().chain_id,
            orderbook_address: sample_ob_id().orderbook_address,
            last_block,
            last_hash: Bytes::from_str("0xbeef").unwrap(),
            updated_at: 1,
        }
    }

    #[tokio::test]
    async fn runner_run_resets_when_tables_missing() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = json!([]);
        let db_meta_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };

        let clear_stmt = clear_tables_stmt();
        let create_stmt = create_tables_stmt();
        let metadata_stmt = insert_db_metadata_stmt(DB_SCHEMA_VERSION);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([db_meta_row]))
            .with_text(&clear_stmt, "ok")
            .with_text(&create_stmt, "ok");
        let db = with_view_creation_sql(db).with_text(&metadata_stmt, "ok");
        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        let mut expected = vec![clear_stmt.sql().to_string(), create_stmt.sql().to_string()];
        expected.extend(
            create_views_batch()
                .statements()
                .iter()
                .map(|stmt| stmt.sql().to_string()),
        );
        expected.push(metadata_stmt.sql().to_string());
        assert_eq!(calls, expected);
    }

    #[tokio::test]
    async fn runner_run_resets_on_missing_db_metadata() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let clear_stmt = clear_tables_stmt();
        let create_stmt = create_tables_stmt();
        let metadata_stmt = insert_db_metadata_stmt(DB_SCHEMA_VERSION);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([])) // triggers reset
            // inspect_state will look for watermark since table exists
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]))
            .with_text(&clear_stmt, "ok")
            .with_text(&create_stmt, "ok");
        let db = with_view_creation_sql(db).with_text(&metadata_stmt, "ok");
        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        assert!(calls.contains(&clear_stmt.sql().to_string()));
        assert!(calls.contains(&create_stmt.sql().to_string()));
        assert!(calls.contains(&metadata_stmt.sql().to_string()));
        for view_stmt in create_views_batch().statements() {
            assert!(calls.contains(&view_stmt.sql().to_string()));
        }
    }

    #[tokio::test]
    async fn runner_run_resets_on_schema_mismatch() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let mismatched_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION + 1,
            created_at: None,
            updated_at: None,
        };

        let clear_stmt = clear_tables_stmt();
        let create_stmt = create_tables_stmt();
        let metadata_stmt = insert_db_metadata_stmt(DB_SCHEMA_VERSION);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]))
            .with_json(&fetch_db_metadata_stmt(), json!([mismatched_row]))
            .with_text(&clear_stmt, "ok")
            .with_text(&create_stmt, "ok");
        let db = with_view_creation_sql(db).with_text(&metadata_stmt, "ok");

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        assert!(calls.contains(&clear_stmt.sql().to_string()));
        assert!(calls.contains(&create_stmt.sql().to_string()));
        assert!(calls.contains(&metadata_stmt.sql().to_string()));
        for view_stmt in create_views_batch().statements() {
            assert!(calls.contains(&view_stmt.sql().to_string()));
        }
    }

    #[tokio::test]
    async fn runner_run_is_idempotent_when_schema_ok() {
        let adapter = ClientBootstrapAdapter::new();
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
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]));

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn runner_run_propagates_unexpected_ensure_schema_error() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();

        let db = MockDb::default().with_json(&fetch_tables_stmt(), tables_json);

        let err = adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap_err();

        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn engine_run_applies_dump_on_fresh_db() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let dump_stmt = SqlStatement::new("--dump-sql");
        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: 100,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_target_watermark_stmt(&cfg.ob_id), json!([]))
            .with_text(&dump_stmt, "ok");

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert_eq!(db.calls(), vec![dump_stmt.sql().to_string()]);
    }

    #[tokio::test]
    async fn engine_run_clears_and_applies_dump_when_threshold_exceeded() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 50_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) + 1;
        let dump_stmt = SqlStatement::new("--dump-sql");
        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: latest,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        let clear_batch = clear_orderbook_data_batch(&sample_ob_id());
        let mut db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.ob_id),
                json!([watermark_row(last_synced)]),
            )
            .with_text(&dump_stmt, "dumped");

        for stmt in clear_batch.statements() {
            db = db.with_text(stmt, "cleared");
        }

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: latest,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        adapter.engine_run(&db, &cfg).await.unwrap();

        let calls = db.calls();
        let mut expected: Vec<String> = clear_batch
            .statements()
            .iter()
            .map(|stmt| stmt.sql().to_string())
            .collect();
        expected.push(dump_stmt.sql().to_string());
        assert_eq!(calls, expected);
    }

    #[tokio::test]
    async fn engine_run_skips_actions_within_threshold() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 100_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) - 1;
        let cfg = cfg_with_dump(latest);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.ob_id),
                json!([watermark_row(last_synced)]),
            );

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn engine_run_skips_actions_at_threshold_boundary() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 120_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD);
        let cfg = cfg_with_dump(latest);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.ob_id),
                json!([watermark_row(last_synced)]),
            );

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn engine_run_does_nothing_without_dump_even_when_threshold_exceeded() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 200_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) + 5;
        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: None,
            latest_block: latest,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.ob_id),
                json!([watermark_row(last_synced)]),
            );

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn engine_run_propagates_clear_errors() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 80_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) + 42;
        let dump_stmt = SqlStatement::new("--dump-sql");
        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: latest,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.ob_id),
                json!([watermark_row(last_synced)]),
            );

        let err = adapter.engine_run(&db, &cfg).await.unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
