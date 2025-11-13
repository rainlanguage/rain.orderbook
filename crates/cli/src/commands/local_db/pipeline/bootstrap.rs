use rain_orderbook_common::local_db::{
    pipeline::adapters::bootstrap::{BootstrapConfig, BootstrapPipeline},
    query::LocalDbQueryExecutor,
    LocalDbError,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ProducerBootstrapAdapter;

impl ProducerBootstrapAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait(?Send)]
impl BootstrapPipeline for ProducerBootstrapAdapter {
    async fn engine_run<DB>(&self, db: &DB, config: &BootstrapConfig) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        self.reset_db(db, None).await?;

        if let Some(dump_stmt) = &config.dump_stmt {
            db.query_text(dump_stmt).await?;
        }

        Ok(())
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
    use alloy::primitives::Address;
    use async_trait::async_trait;
    use rain_orderbook_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
    use rain_orderbook_common::local_db::query::clear_tables::clear_tables_stmt;
    use rain_orderbook_common::local_db::query::create_tables::create_tables_stmt;
    use rain_orderbook_common::local_db::query::create_views::create_views_batch;
    use rain_orderbook_common::local_db::query::insert_db_metadata::insert_db_metadata_stmt;
    use rain_orderbook_common::local_db::query::{
        FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    };
    use rain_orderbook_common::local_db::OrderbookIdentifier;

    const TEST_BLOCK_NUMBER_THRESHOLD: u32 = 10_000;

    #[derive(Default)]
    struct MockDb {
        text_map: HashMap<String, String>,
        calls_text: Mutex<Vec<String>>,
    }

    impl MockDb {
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

        async fn query_json<T>(&self, _stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            Err(LocalDbQueryError::database("not supported in these tests"))
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
        OrderbookIdentifier::new(1, Address::ZERO)
    }

    #[tokio::test]
    async fn engine_run_resets_and_does_not_import_when_no_dump() {
        let adapter = ProducerBootstrapAdapter::new();
        let clear_stmt = clear_tables_stmt();
        let create_stmt = create_tables_stmt();
        let metadata_stmt = insert_db_metadata_stmt(DB_SCHEMA_VERSION);
        let db = MockDb::default()
            .with_text(&clear_stmt, "ok")
            .with_text(&create_stmt, "ok");
        let db = with_view_creation_sql(db).with_text(&metadata_stmt, "ok");

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: None,
            latest_block: 0,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        adapter.engine_run(&db, &cfg).await.unwrap();

        let calls = db.calls();
        // Presence assertions
        let clear = clear_stmt.sql().to_string();
        let create = create_stmt.sql().to_string();
        let insert = metadata_stmt.sql().to_string();
        let view_stmts: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|stmt| stmt.sql().to_string())
            .collect();

        assert!(calls.contains(&clear));
        assert!(calls.contains(&create));
        assert!(calls.contains(&insert));
        for view in &view_stmts {
            assert!(calls.contains(view));
        }

        // Ordering: clear -> create -> views -> insert
        let idx = |s: &String| calls.iter().position(|c| c == s).unwrap();
        assert!(idx(&clear) < idx(&create));
        for view in &view_stmts {
            assert!(idx(&create) < idx(view));
        }
        let last_view_idx = view_stmts.iter().map(&idx).max().expect("views present");
        assert!(last_view_idx < idx(&insert));
    }

    #[tokio::test]
    async fn engine_run_resets_and_imports_dump_when_present() {
        let adapter = ProducerBootstrapAdapter::new();
        let dump_stmt = SqlStatement::new("--dump-sql");
        let clear_stmt = clear_tables_stmt();
        let create_stmt = create_tables_stmt();
        let metadata_stmt = insert_db_metadata_stmt(DB_SCHEMA_VERSION);
        let db = MockDb::default()
            .with_text(&clear_stmt, "ok")
            .with_text(&create_stmt, "ok");
        let db = with_view_creation_sql(db)
            .with_text(&metadata_stmt, "ok")
            .with_text(&dump_stmt, "ok");

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: 0,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        adapter.engine_run(&db, &cfg).await.unwrap();

        let calls = db.calls();
        // Presence assertions
        let clear = clear_stmt.sql().to_string();
        let create = create_stmt.sql().to_string();
        let insert = metadata_stmt.sql().to_string();
        let dump = dump_stmt.sql().to_string();
        let view_stmts: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|stmt| stmt.sql().to_string())
            .collect();

        assert!(calls.contains(&clear));
        assert!(calls.contains(&create));
        assert!(calls.contains(&insert));
        assert!(calls.contains(&dump));
        for view in &view_stmts {
            assert!(calls.contains(view));
        }

        // Ordering: clear -> create -> views -> insert -> dump
        let idx = |s: &String| calls.iter().position(|c| c == s).unwrap();
        assert!(idx(&clear) < idx(&create));
        for view in &view_stmts {
            assert!(idx(&create) < idx(view));
        }
        let last_view_idx = view_stmts.iter().map(&idx).max().expect("views present");
        assert!(last_view_idx < idx(&insert));
        assert!(idx(&insert) < idx(&dump));
    }

    #[tokio::test]
    async fn engine_run_resets_and_fails_when_dump_missing() {
        let adapter = ProducerBootstrapAdapter::new();
        let dump_stmt = SqlStatement::new("--dump-sql-missing");
        let clear_stmt = clear_tables_stmt();
        let create_stmt = create_tables_stmt();
        let metadata_stmt = insert_db_metadata_stmt(DB_SCHEMA_VERSION);
        let db = MockDb::default()
            .with_text(&clear_stmt, "ok")
            .with_text(&create_stmt, "ok");
        let db = with_view_creation_sql(db).with_text(&metadata_stmt, "ok");

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: 0,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        // Expect error due to missing dump mapping, after successful reset
        let result = adapter.engine_run(&db, &cfg).await;
        assert!(result.is_err());

        let calls = db.calls();
        let clear = clear_stmt.sql().to_string();
        let create = create_stmt.sql().to_string();
        let insert = metadata_stmt.sql().to_string();
        let dump = dump_stmt.sql().to_string();
        let view_stmts: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|stmt| stmt.sql().to_string())
            .collect();

        assert!(calls.contains(&clear));
        assert!(calls.contains(&create));
        assert!(calls.contains(&insert));
        assert!(calls.contains(&dump));
        for view in &view_stmts {
            assert!(calls.contains(view));
        }

        // Ordering: clear -> create -> views -> insert -> dump (dump last attempted and fails)
        let idx = |s: &String| calls.iter().position(|c| c == s).unwrap();
        assert!(idx(&clear) < idx(&create));
        for view in &view_stmts {
            assert!(idx(&create) < idx(view));
        }
        let last_view_idx = view_stmts.iter().map(&idx).max().expect("views present");
        assert!(last_view_idx < idx(&insert));
        assert!(idx(&insert) < idx(&dump));
    }

    #[tokio::test]
    async fn engine_run_propagates_reset_error() {
        let adapter = ProducerBootstrapAdapter::new();
        let db = MockDb::default().with_text(&clear_tables_stmt(), "ok");

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: None,
            latest_block: 0,
            block_number_threshold: 1,
        };

        let err = adapter.engine_run(&db, &cfg).await.unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }

        let calls = db.calls();
        assert!(calls.contains(&clear_tables_stmt().sql().to_string()));
        assert!(calls.contains(&create_tables_stmt().sql().to_string()));
    }

    #[tokio::test]
    async fn runner_run_is_unimplemented() {
        let adapter = ProducerBootstrapAdapter::new();
        let db = MockDb::default();

        let err = adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap_err();
        match err {
            LocalDbError::InvalidBootstrapImplementation => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
