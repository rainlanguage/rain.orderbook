use crate::local_db::pipeline::traits::BootstrapConfig;
use crate::local_db::pipeline::traits::BootstrapPipeline;
use crate::local_db::pipeline::traits::BootstrapState;
use crate::local_db::pipeline::traits::TargetKey;
use crate::local_db::query::clear_orderbook_data::clear_orderbook_data_stmt;
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
/// BootstrapPipeline trait. Environment runners can provide
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

    async fn clear_orderbook_data<DB>(
        &self,
        db: &DB,
        target: &TargetKey,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        db.query_text(&clear_orderbook_data_stmt(
            target.chain_id,
            target.orderbook_address,
        ))
        .await?;
        Ok(())
    }

    async fn engine_run<DB>(&self, _: &DB, _: &BootstrapConfig) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        Err(LocalDbError::InvalidBootstrapImplementation)
    }

    async fn runner_run<DB>(&self, _: &DB, _: Option<u32>) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        Err(LocalDbError::InvalidBootstrapImplementation)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use crate::local_db::query::fetch_db_metadata::{fetch_db_metadata_stmt, DbMetadataRow};
    use crate::local_db::query::fetch_tables::{fetch_tables_stmt, TableResponse};
    use crate::local_db::query::{FromDbJson, LocalDbQueryError, SqlStatement, SqlStatementBatch};
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

    struct RecordingTextExecutor {
        result: Mutex<Option<Result<(), LocalDbError>>>,
        captured_sql: Mutex<Vec<String>>,
    }

    impl RecordingTextExecutor {
        fn new(result: Result<(), LocalDbError>) -> Self {
            Self {
                result: Mutex::new(Some(result)),
                captured_sql: Mutex::new(Vec::new()),
            }
        }

        fn succeed() -> Self {
            Self::new(Ok(()))
        }

        fn fail(err: LocalDbError) -> Self {
            Self::new(Err(err))
        }

        fn captured_sql(&self) -> Vec<String> {
            self.captured_sql.lock().unwrap().clone()
        }
    }

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for RecordingTextExecutor {
        async fn execute_batch(&self, _: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            panic!("execute_batch should not be called in these tests");
        }

        async fn query_json<T>(&self, _: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            panic!("query_json should not be called in these tests");
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            self.captured_sql
                .lock()
                .unwrap()
                .push(stmt.sql().to_string());

            let outcome = self
                .result
                .lock()
                .unwrap()
                .take()
                .expect("query_text called more than expected");

            match outcome {
                Ok(()) => Ok(String::from("ok")),
                Err(LocalDbError::LocalDbQueryError(inner)) => Err(inner),
                Err(err) => Err(LocalDbQueryError::database(err.to_string())),
            }
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

    #[tokio::test]
    async fn ensure_schema_ok_with_matching_version() {
        let adapter = DefaultBootstrapAdapter::new();
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DATABASE_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default().with_json(&fetch_db_metadata_stmt(), json!([db_row]));

        adapter.ensure_schema(&db, None).await.expect("schema ok");
    }

    #[tokio::test]
    async fn ensure_schema_err_on_mismatch() {
        let adapter = DefaultBootstrapAdapter::new();
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DATABASE_SCHEMA_VERSION + 1,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default().with_json(&fetch_db_metadata_stmt(), json!([db_row]));

        let err = adapter.ensure_schema(&db, None).await.unwrap_err();
        match err {
            LocalDbError::SchemaVersionMismatch { expected, found } => {
                assert_eq!(expected, DATABASE_SCHEMA_VERSION);
                assert_eq!(found, DATABASE_SCHEMA_VERSION + 1);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn ensure_schema_honors_override_ok() {
        let adapter = DefaultBootstrapAdapter::new();
        let override_version = DATABASE_SCHEMA_VERSION + 7;
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: override_version,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default().with_json(&fetch_db_metadata_stmt(), json!([db_row]));

        adapter
            .ensure_schema(&db, Some(override_version))
            .await
            .expect("schema ok with override");
    }

    #[tokio::test]
    async fn ensure_schema_honors_override_mismatch() {
        let adapter = DefaultBootstrapAdapter::new();
        let row_version = DATABASE_SCHEMA_VERSION + 3;
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: row_version,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default().with_json(&fetch_db_metadata_stmt(), json!([db_row]));

        let err = adapter
            .ensure_schema(&db, Some(DATABASE_SCHEMA_VERSION))
            .await
            .unwrap_err();
        match err {
            LocalDbError::SchemaVersionMismatch { expected, found } => {
                assert_eq!(expected, DATABASE_SCHEMA_VERSION);
                assert_eq!(found, row_version);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn ensure_schema_err_on_missing_row() {
        let adapter = DefaultBootstrapAdapter::new();
        let db = MockDb::default().with_json(&fetch_db_metadata_stmt(), json!([]));
        let err = adapter.ensure_schema(&db, None).await.unwrap_err();
        match err {
            LocalDbError::MissingDbMetadataRow => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn inspect_state_tables_and_last_synced_block() {
        let adapter = DefaultBootstrapAdapter::new();
        // Provide all required tables
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        // Watermark row present
        let target_key = TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        };
        let watermark_stmt =
            fetch_target_watermark_stmt(target_key.chain_id, target_key.orderbook_address);
        let watermark_json = json!([TargetWatermarkRow {
            chain_id: target_key.chain_id,
            orderbook_address: target_key.orderbook_address,
            last_block: 123,
            last_hash: None,
            updated_at: None,
        }]);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&watermark_stmt, watermark_json);

        let state = adapter.inspect_state(&db, &target_key).await.unwrap();
        assert!(state.has_required_tables);
        assert_eq!(state.last_synced_block, Some(123));
    }

    #[tokio::test]
    async fn inspect_state_missing_tables_means_not_ready_and_no_watermark_query() {
        let adapter = DefaultBootstrapAdapter::new();
        let db = MockDb::default().with_json(&fetch_tables_stmt(), json!([]));
        let target_key = TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        };
        let state = adapter.inspect_state(&db, &target_key).await.unwrap();
        assert!(!state.has_required_tables);
        assert_eq!(state.last_synced_block, None);
    }

    #[tokio::test]
    async fn inspect_state_missing_only_watermark_table() {
        let adapter = DefaultBootstrapAdapter::new();
        // All required tables except `target_watermarks`.
        let names: Vec<&str> = REQUIRED_TABLES
            .iter()
            .copied()
            .filter(|t| !t.eq_ignore_ascii_case("target_watermarks"))
            .collect();
        assert!(names.len() + 1 == REQUIRED_TABLES.len());
        let tables_json = serde_json::to_value(
            names
                .into_iter()
                .map(|name| TableResponse {
                    name: name.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let db = MockDb::default().with_json(&fetch_tables_stmt(), tables_json);
        let target_key = TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        };
        let state = adapter.inspect_state(&db, &target_key).await.unwrap();
        assert!(!state.has_required_tables);
        assert_eq!(state.last_synced_block, None);
    }

    #[tokio::test]
    async fn inspect_state_watermark_table_present_but_empty() {
        let adapter = DefaultBootstrapAdapter::new();
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let target_key = TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        };
        let watermark_stmt =
            fetch_target_watermark_stmt(target_key.chain_id, target_key.orderbook_address);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&watermark_stmt, json!([]));

        let state = adapter.inspect_state(&db, &target_key).await.unwrap();
        assert!(state.has_required_tables);
        assert_eq!(state.last_synced_block, None);
    }

    #[tokio::test]
    async fn inspect_state_table_names_case_insensitive() {
        let adapter = DefaultBootstrapAdapter::new();
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .enumerate()
                .map(|(i, &name)| {
                    let cased = if i % 2 == 0 {
                        name.to_ascii_uppercase()
                    } else {
                        name.to_ascii_lowercase()
                    };
                    TableResponse { name: cased }
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let target_key = TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        };
        let watermark_stmt =
            fetch_target_watermark_stmt(target_key.chain_id, target_key.orderbook_address);
        let watermark_json = json!([TargetWatermarkRow {
            chain_id: target_key.chain_id,
            orderbook_address: target_key.orderbook_address,
            last_block: 42,
            last_hash: None,
            updated_at: None,
        }]);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&watermark_stmt, watermark_json);

        let state = adapter.inspect_state(&db, &target_key).await.unwrap();
        assert!(state.has_required_tables);
        assert_eq!(state.last_synced_block, Some(42));
    }

    #[tokio::test]
    async fn inspect_state_propagates_fetch_tables_error() {
        let adapter = DefaultBootstrapAdapter::new();
        let db = MockDb::default(); // no json for fetch_tables_stmt()
        let target_key = TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        };
        let err = adapter.inspect_state(&db, &target_key).await.unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn inspect_state_propagates_watermark_query_error() {
        let adapter = DefaultBootstrapAdapter::new();
        // Include all required tables so it attempts the watermark query.
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let db = MockDb::default().with_json(&fetch_tables_stmt(), tables_json);
        let target_key = TargetKey {
            chain_id: 1,
            orderbook_address: Address::ZERO,
        };
        // Intentionally do not provide json for watermark_stmt -> should error
        let err = adapter.inspect_state(&db, &target_key).await.unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn reset_db_runs_clear_create_and_insert() {
        let adapter = DefaultBootstrapAdapter::new();
        let db = MockDb::default()
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok");

        adapter
            .reset_db(&db, Some(DATABASE_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        assert_eq!(calls.len(), 3);
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
    async fn reset_db_uses_default_version_when_none() {
        let adapter = DefaultBootstrapAdapter::new();
        let db = MockDb::default()
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok");

        adapter.reset_db(&db, None).await.unwrap();

        let calls = db.calls();
        assert_eq!(calls.len(), 3);
        assert_eq!(
            calls[2],
            insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION).sql()
        );
    }

    #[tokio::test]
    async fn reset_db_uses_custom_version_when_some() {
        let adapter = DefaultBootstrapAdapter::new();
        let custom_version = DATABASE_SCHEMA_VERSION + 9;
        let db = MockDb::default()
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(custom_version), "ok");

        adapter.reset_db(&db, Some(custom_version)).await.unwrap();

        let calls = db.calls();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[2], insert_db_metadata_stmt(custom_version).sql());
    }

    #[tokio::test]
    async fn reset_db_propagates_errors() {
        let adapter = DefaultBootstrapAdapter::new();
        // Only the first statement is present; second will fail.
        let db = MockDb::default().with_text(&clear_tables_stmt(), "ok");

        let err = adapter.reset_db(&db, None).await.unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }

        let calls = db.calls();
        assert_eq!(calls.len(), 2); // attempted clear and create
        assert_eq!(calls[0], clear_tables_stmt().sql());
        assert_eq!(calls[1], create_tables_stmt().sql());
    }

    #[tokio::test]
    async fn engine_run_returns_invalid_bootstrap_implementation() {
        let adapter = DefaultBootstrapAdapter::new();
        let db = MockDb::default();
        let cfg = BootstrapConfig {
            target_key: TargetKey {
                chain_id: 1,
                orderbook_address: Address::ZERO,
            },
            dump_stmt: None,
            latest_block: 0,
        };

        let err = adapter.engine_run(&db, &cfg).await.unwrap_err();
        match err {
            LocalDbError::InvalidBootstrapImplementation => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn runner_run_returns_invalid_bootstrap_implementation() {
        let adapter = DefaultBootstrapAdapter::new();
        let db = MockDb::default();

        let err = adapter.runner_run(&db, None).await.unwrap_err();
        match err {
            LocalDbError::InvalidBootstrapImplementation => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn clear_orderbook_data_executes_expected_statement() {
        let adapter = DefaultBootstrapAdapter::new();
        let target = TargetKey {
            chain_id: 42161,
            orderbook_address: Address::from([0x11; 20]),
        };
        let db = RecordingTextExecutor::succeed();

        adapter
            .clear_orderbook_data(&db, &target)
            .await
            .expect("clear_orderbook_data should succeed");

        let captured = db.captured_sql();
        assert_eq!(captured.len(), 1);
        assert_eq!(
            captured[0],
            clear_orderbook_data_stmt(target.chain_id, target.orderbook_address).sql()
        );
    }

    #[tokio::test]
    async fn clear_orderbook_data_propagates_error() {
        let adapter = DefaultBootstrapAdapter::new();
        let target = TargetKey {
            chain_id: 10,
            orderbook_address: Address::from([0x22; 20]),
        };
        let inner_error = LocalDbQueryError::database("boom");
        let db = RecordingTextExecutor::fail(LocalDbError::from(inner_error.clone()));

        let err = adapter
            .clear_orderbook_data(&db, &target)
            .await
            .expect_err("clear_orderbook_data should propagate error");

        match err {
            LocalDbError::LocalDbQueryError(actual) => {
                assert_eq!(actual.to_string(), inner_error.to_string());
            }
            other => panic!("unexpected error: {other:?}"),
        }

        let captured = db.captured_sql();
        assert_eq!(captured.len(), 1);
        assert_eq!(
            captured[0],
            clear_orderbook_data_stmt(target.chain_id, target.orderbook_address).sql()
        );
    }
}
