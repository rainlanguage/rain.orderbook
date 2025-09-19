use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

/// Execute an arbitrary SQL script against the provided SQLite database.
pub fn sqlite_execute(db_path: &str, sql_script: &str) -> Result<()> {
    let mut child = Command::new("sqlite3")
        .arg(db_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn sqlite3 command")?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to open sqlite3 stdin"))?;
        stdin
            .write_all(sql_script.as_bytes())
            .with_context(|| "Failed to write SQL script to sqlite3")?;
    }

    let output = child
        .wait_with_output()
        .with_context(|| "Failed to wait for sqlite3 command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "sqlite3 exited with status {}: {}",
            output.status,
            stderr
        ));
    }

    Ok(())
}

/// Run a query and deserialize the JSON output into `T`.
pub fn sqlite_query_json<T>(db_path: &str, sql: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let output = Command::new("sqlite3")
        .arg("-json")
        .arg(db_path)
        .arg(sql)
        .output()
        .with_context(|| "Failed to run sqlite3 query")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "sqlite3 exited with status {} for query {}: {}",
            output.status,
            sql,
            stderr
        ));
    }

    let stdout =
        String::from_utf8(output.stdout).with_context(|| "sqlite3 produced non-UTF8 output")?;
    let trimmed = stdout.trim();
    let payload = if trimmed.is_empty() { "[]" } else { trimmed };

    serde_json::from_str(payload).with_context(|| "Failed to decode sqlite3 JSON output")
}

#[derive(serde::Deserialize)]
struct TableNameRow {
    name: String,
}

/// Return the set of tables currently present in the DB.
pub fn sqlite_list_tables(db_path: &str) -> Result<HashSet<String>> {
    const TABLE_QUERY: &str =
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';";
    let rows: Vec<TableNameRow> = sqlite_query_json(db_path, TABLE_QUERY)?;
    Ok(rows
        .into_iter()
        .map(|row| row.name.to_ascii_lowercase())
        .collect())
}

/// Do all `required_tables` exist in the DB?
pub fn sqlite_has_required_tables(db_path: &str, required_tables: &[&str]) -> Result<bool> {
    let existing = sqlite_list_tables(db_path)?;
    Ok(required_tables
        .iter()
        .all(|table| existing.contains(&table.to_ascii_lowercase())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn execute_and_query_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(
            &db_path_str,
            "CREATE TABLE numbers (n INTEGER); INSERT INTO numbers (n) VALUES (1), (2);",
        )
        .unwrap();

        #[derive(serde::Deserialize)]
        struct CountRow {
            c: i64,
        }
        let rows: Vec<CountRow> =
            sqlite_query_json(&db_path_str, "SELECT COUNT(*) AS c FROM numbers;").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].c, 2);
    }

    #[test]
    fn detects_required_tables() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("schema.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(
            &db_path_str,
            "CREATE TABLE foo (id INTEGER); CREATE TABLE bar (id INTEGER);",
        )
        .unwrap();

        assert!(sqlite_has_required_tables(&db_path_str, &["foo", "bar"]).unwrap());
        assert!(!sqlite_has_required_tables(&db_path_str, &["foo", "baz"]).unwrap());
    }
}
