use async_trait::async_trait;
use rain_orderbook_common::local_db::query::{FromDbJson, LocalDbQueryError, LocalDbQueryExecutor};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct SqliteCliExecutor {
    db_path: PathBuf,
}

impl SqliteCliExecutor {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait(?Send)]
impl LocalDbQueryExecutor for SqliteCliExecutor {
    async fn query_text(&self, sql: &str) -> Result<String, LocalDbQueryError> {
        let mut child = Command::new("sqlite3")
            .arg(self.db_path.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| LocalDbQueryError::database(format!("Failed to spawn sqlite3: {e}")))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(sql.as_bytes())
                .map_err(|e| LocalDbQueryError::database(format!("Failed to write SQL: {e}")))?;
        } else {
            return Err(LocalDbQueryError::database("Failed to open sqlite3 stdin"));
        }

        let output = child
            .wait_with_output()
            .map_err(|e| LocalDbQueryError::database(format!("sqlite3 wait failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LocalDbQueryError::database(format!(
                "sqlite3 exited with status {}: {}",
                output.status, stderr
            )));
        }
        Ok(String::new())
    }

    async fn query_json<T>(&self, sql: &str) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson,
    {
        let output = Command::new("sqlite3")
            .arg("-json")
            .arg(self.db_path.as_os_str())
            .arg(sql)
            .output()
            .map_err(|e| LocalDbQueryError::database(format!("Failed to run sqlite3: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LocalDbQueryError::database(format!(
                "sqlite3 exited with status {} for query {}: {}",
                output.status, sql, stderr
            )));
        }

        let stdout =
            String::from_utf8(output.stdout).map_err(|_| LocalDbQueryError::invalid_response())?;
        let trimmed = stdout.trim();
        let payload = if trimmed.is_empty() { "[]" } else { trimmed };

        serde_json::from_str::<T>(payload)
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

        let exec = SqliteCliExecutor::new(&*db_path_str);
        exec.query_text(
            "CREATE TABLE numbers (n INTEGER); INSERT INTO numbers (n) VALUES (1), (2);",
        )
        .await
        .unwrap();

        #[derive(serde::Deserialize)]
        struct CountRow {
            c: i64,
        }
        let rows: Vec<CountRow> = exec
            .query_json("SELECT COUNT(*) AS c FROM numbers;")
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

        let exec = SqliteCliExecutor::new(&*db_path_str);
        exec.query_text("CREATE TABLE foo (id INTEGER); CREATE TABLE bar (id INTEGER);")
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct TableNameRow {
            name: String,
        }
        const TABLE_QUERY: &str =
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';";
        let rows: Vec<TableNameRow> = exec.query_json(TABLE_QUERY).await.unwrap();
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
}
