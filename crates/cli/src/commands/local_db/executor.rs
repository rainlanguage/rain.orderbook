use async_trait::async_trait;
use rain_orderbook_common::local_db::query::{
    FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlValue,
};
use rusqlite::{types::ValueRef, Connection};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct RusqliteExecutor {
    db_path: PathBuf,
}

fn sqlvalue_to_rusqlite(v: SqlValue) -> rusqlite::types::Value {
    match v {
        SqlValue::Text(t) => rusqlite::types::Value::Text(t),
        SqlValue::I64(i) => rusqlite::types::Value::Integer(i),
        SqlValue::Null => rusqlite::types::Value::Null,
    }
}

impl RusqliteExecutor {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait(?Send)]
impl LocalDbQueryExecutor for RusqliteExecutor {
    async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| LocalDbQueryError::database(format!("Failed to open database: {e}")))?;
        conn.busy_timeout(Duration::from_millis(500))
            .map_err(|e| LocalDbQueryError::database(format!("Failed to set busy_timeout: {e}")))?;
        if stmt.params().is_empty() {
            conn.execute_batch(stmt.sql())
                .map_err(|e| LocalDbQueryError::database(format!("SQL execution failed: {e}")))?;
            Ok(String::new())
        } else {
            let mut s = conn.prepare(stmt.sql()).map_err(|e| {
                LocalDbQueryError::database(format!("Failed to prepare query: {e}"))
            })?;
            let bound = stmt.params().iter().cloned().map(sqlvalue_to_rusqlite);
            let params = rusqlite::params_from_iter(bound);
            s.execute(params)
                .map_err(|e| LocalDbQueryError::database(format!("SQL execution failed: {e}")))?;
            Ok(String::new())
        }
    }

    async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson,
    {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| LocalDbQueryError::database(format!("Failed to open database: {e}")))?;
        conn.busy_timeout(Duration::from_millis(500))
            .map_err(|e| LocalDbQueryError::database(format!("Failed to set busy_timeout: {e}")))?;

        let mut s = conn
            .prepare(stmt.sql())
            .map_err(|e| LocalDbQueryError::database(format!("Failed to prepare query: {e}")))?;
        let column_names: Vec<String> = (0..s.column_count())
            .map(|i| {
                let raw = s.column_name(i).unwrap_or("");
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    format!("column_{}", i)
                } else {
                    trimmed.to_string()
                }
            })
            .collect();

        let bound = stmt.params().iter().cloned().map(sqlvalue_to_rusqlite);
        let params = rusqlite::params_from_iter(bound);

        let rows_iter = s
            .query_map(params, |row| {
                let mut obj = Map::with_capacity(column_names.len());
                for (i, name) in column_names.iter().enumerate() {
                    let v = match row.get_ref(i)? {
                        ValueRef::Null => Value::Null,
                        ValueRef::Integer(n) => json!(n),
                        ValueRef::Real(f) => json!(f),
                        ValueRef::Text(bytes) => match std::str::from_utf8(bytes) {
                            Ok(s) => json!(s),
                            Err(_) => json!(alloy::hex::encode_prefixed(bytes)),
                        },
                        ValueRef::Blob(bytes) => json!(alloy::hex::encode_prefixed(bytes)),
                    };
                    obj.insert(name.clone(), v);
                }
                Ok(Value::Object(obj))
            })
            .map_err(|e| LocalDbQueryError::database(format!("Query failed: {e}")))?;

        let mut out: Vec<Value> = Vec::new();
        for r in rows_iter {
            let v = r.map_err(|e| LocalDbQueryError::database(format!("Row error: {e}")))?;
            out.push(v);
        }

        let json_value = Value::Array(out);
        serde_json::from_value::<T>(json_value)
            .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn execute_and_query_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(
            "CREATE TABLE numbers (n INTEGER); INSERT INTO numbers (n) VALUES (1), (2);",
        ))
        .await
        .unwrap();

        #[derive(serde::Deserialize)]
        struct CountRow {
            c: i64,
        }
        let rows: Vec<CountRow> = exec
            .query_json(&SqlStatement::new("SELECT COUNT(*) AS c FROM numbers;"))
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].c, 2);
    }

    #[tokio::test]
    async fn detects_required_tables() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("schema.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(
            "CREATE TABLE foo (id INTEGER); CREATE TABLE bar (id INTEGER);",
        ))
        .await
        .unwrap();

        #[derive(serde::Deserialize)]
        struct TableNameRow {
            name: String,
        }
        const TABLE_QUERY: &str =
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';";
        let rows: Vec<TableNameRow> = exec
            .query_json(&SqlStatement::new(TABLE_QUERY))
            .await
            .unwrap();
        let existing: std::collections::HashSet<String> = rows
            .into_iter()
            .map(|row| row.name.to_ascii_lowercase())
            .collect();

        let has = |req: &[&str]| {
            req.iter()
                .all(|t| existing.contains(&t.to_ascii_lowercase()))
        };
        assert!(has(&["foo", "bar"]));
        assert!(!has(&["foo", "baz"]));
    }

    #[tokio::test]
    async fn query_json_multi_column_rows() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("multi.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(
            r#"
                CREATE TABLE people (
                    id INTEGER,
                    name TEXT,
                    note BLOB
                );
                INSERT INTO people (id, name, note) VALUES
                    (1, 'Alice', X'000102'),
                    (2, 'Bob',   X'ff'),
                    (3, 'Carol', NULL),
                    (4, 'Дора',  X'c0'),
                    (5, 'Eve',   X'');
                "#,
        ))
        .await
        .unwrap();

        #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
        struct PersonRow {
            id: i64,
            name: String,
            note: Option<String>,
        }

        let rows: Vec<PersonRow> = exec
            .query_json(&SqlStatement::new(
                "SELECT id, name, note FROM people ORDER BY id ASC;",
            ))
            .await
            .unwrap();

        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows,
            vec![
                PersonRow {
                    id: 1,
                    name: "Alice".to_string(),
                    note: Some("0x000102".to_string()),
                },
                PersonRow {
                    id: 2,
                    name: "Bob".to_string(),
                    note: Some("0xff".to_string()),
                },
                PersonRow {
                    id: 3,
                    name: "Carol".to_string(),
                    note: None,
                },
                PersonRow {
                    id: 4,
                    name: "Дора".to_string(),
                    note: Some("0xc0".to_string()),
                },
                PersonRow {
                    id: 5,
                    name: "Eve".to_string(),
                    note: Some("0x".to_string()),
                },
            ]
        );
    }
}
