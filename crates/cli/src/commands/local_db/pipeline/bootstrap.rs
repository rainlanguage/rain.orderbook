use rain_orderbook_common::local_db::{
    pipeline::{BootstrapConfig, BootstrapPipeline},
    query::LocalDbQueryExecutor,
    LocalDbError,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ProducerBootstrapAdapter;

impl ProducerBootstrapAdapter {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait(?Send)]
impl BootstrapPipeline for ProducerBootstrapAdapter {
    async fn run<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
        config: &BootstrapConfig,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        self.reset_db(db, db_schema_version).await?;

        if let Some(dump_stmt) = &config.dump_stmt {
            db.query_text(dump_stmt).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use alloy::primitives::Address;
    use async_trait::async_trait;
    use rain_orderbook_common::local_db::pipeline::BootstrapConfig;
    use rain_orderbook_common::local_db::query::clear_tables::clear_tables_stmt;
    use rain_orderbook_common::local_db::query::create_tables::create_tables_stmt;
    use rain_orderbook_common::local_db::query::insert_db_metadata::insert_db_metadata_stmt;
    use rain_orderbook_common::local_db::query::{
        FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    };
    use rain_orderbook_common::local_db::{OrderbookIdentifier, DATABASE_SCHEMA_VERSION};

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
    async fn run_resets_and_does_not_import_when_no_dump() {
        let adapter = ProducerBootstrapAdapter::new();
        let db = MockDb::default()
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok");

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: None,
            latest_block: 0,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        // Presence assertions
        let clear = clear_tables_stmt().sql().to_string();
        let create = create_tables_stmt().sql().to_string();
        let insert = insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION)
            .sql()
            .to_string();

        assert!(calls.contains(&clear));
        assert!(calls.contains(&create));
        assert!(calls.contains(&insert));

        // Ordering: clear -> create -> insert
        let idx = |s: &String| calls.iter().position(|c| c == s).unwrap();
        assert!(idx(&clear) < idx(&create));
        assert!(idx(&create) < idx(&insert));
    }

    #[tokio::test]
    async fn run_resets_and_imports_dump_when_present() {
        let adapter = ProducerBootstrapAdapter::new();
        let dump_stmt = SqlStatement::new("--dump-sql");
        let db = MockDb::default()
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok")
            .with_text(&dump_stmt, "ok");

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: 0,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        adapter
            .run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg)
            .await
            .unwrap();

        let calls = db.calls();
        // Presence assertions
        let clear = clear_tables_stmt().sql().to_string();
        let create = create_tables_stmt().sql().to_string();
        let insert = insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION)
            .sql()
            .to_string();
        let dump = dump_stmt.sql().to_string();

        assert!(calls.contains(&clear));
        assert!(calls.contains(&create));
        assert!(calls.contains(&insert));
        assert!(calls.contains(&dump));

        // Ordering: clear -> create -> insert -> dump
        let idx = |s: &String| calls.iter().position(|c| c == s).unwrap();
        assert!(idx(&clear) < idx(&create));
        assert!(idx(&create) < idx(&insert));
        assert!(idx(&insert) < idx(&dump));
    }

    #[tokio::test]
    async fn run_resets_and_fails_when_dump_missing() {
        let adapter = ProducerBootstrapAdapter::new();
        let dump_stmt = SqlStatement::new("--dump-sql-missing");
        let db = MockDb::default()
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
            .with_text(&insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION), "ok");

        let cfg = BootstrapConfig {
            ob_id: sample_ob_id(),
            dump_stmt: Some(dump_stmt.clone()),
            latest_block: 0,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
        };

        // Expect error due to missing dump mapping, after successful reset
        let result = adapter.run(&db, Some(DATABASE_SCHEMA_VERSION), &cfg).await;
        assert!(result.is_err());

        let calls = db.calls();
        let clear = clear_tables_stmt().sql().to_string();
        let create = create_tables_stmt().sql().to_string();
        let insert = insert_db_metadata_stmt(DATABASE_SCHEMA_VERSION)
            .sql()
            .to_string();
        let dump = dump_stmt.sql().to_string();

        assert!(calls.contains(&clear));
        assert!(calls.contains(&create));
        assert!(calls.contains(&insert));
        assert!(calls.contains(&dump));

        // Ordering: clear -> create -> insert -> dump (dump last attempted and fails)
        let idx = |s: &String| calls.iter().position(|c| c == s).unwrap();
        assert!(idx(&clear) < idx(&create));
        assert!(idx(&create) < idx(&insert));
        assert!(idx(&insert) < idx(&dump));
    }
}
