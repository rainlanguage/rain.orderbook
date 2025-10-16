use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
pub struct DbDump {
    #[clap(long, value_name = "PATH", action = clap::ArgAction::Append)]
    pub data_sql: Vec<String>,
    #[clap(long)]
    pub table_schema_file: String,
    #[clap(long)]
    pub end_block: u64,
    #[clap(long)]
    pub db_path: Option<String>,
    #[clap(long)]
    pub dump_file_path: Option<String>,
}

impl DbDump {
    pub async fn execute(self) -> Result<()> {
        let DbDump {
            data_sql,
            table_schema_file,
            end_block,
            db_path,
            dump_file_path,
        } = self;

        let data_sql_files = data_sql;
        let view_sql_files = resolve_view_sql_files()?;
        let db_path =
            db_path.unwrap_or_else(|| format!("src/commands/local_db/local_db_{}.db", end_block));
        let dump_file_path = dump_file_path
            .unwrap_or_else(|| format!("src/commands/local_db/local_db_{}.sql", end_block));

        if let Some(parent) = Path::new(&db_path).parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = Path::new(&dump_file_path).parent() {
            fs::create_dir_all(parent)?;
        }

        let _ = fs::remove_file(&db_path);

        let tables_sql_path = &table_schema_file;

        let _ = Command::new("sqlite3")
            .arg(&db_path)
            .arg(format!(".read {}", tables_sql_path))
            .status()?;

        if data_sql_files.is_empty() {
            eprintln!("warning: no --data-sql files provided; creating schema-only DB dump");
        } else {
            for file in data_sql_files {
                let _ = Command::new("sqlite3")
                    .arg(&db_path)
                    .arg(format!(".read {}", file))
                    .status()?;
            }
        }

        for file in view_sql_files {
            let _ = Command::new("sqlite3")
                .arg(&db_path)
                .arg(format!(".read {}", file))
                .status()?;
        }

        let output = Command::new("sqlite3")
            .arg(&db_path)
            .arg(".dump")
            .output()?;

        fs::write(&dump_file_path, output.stdout)?;

        // Gzip the SQL dump but keep the original .sql alongside the .gz
        Command::new("gzip")
            .arg("-kf")
            .arg(&dump_file_path)
            .status()?;

        Ok(())
    }
}

fn resolve_view_sql_files() -> Result<Vec<String>> {
    collect_sql_files(&default_views_dir())
}

fn default_views_dir() -> PathBuf {
    PathBuf::from("../common/src/raindex_client/local_db/views")
}

fn collect_sql_files(dir: &Path) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut sql_paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("sql") {
            sql_paths.push(path);
        }
    }

    sql_paths.sort();

    Ok(sql_paths
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const TEST_TABLE_SCHEMA: &str = r#"
CREATE TABLE test_orders (
    id INTEGER PRIMARY KEY,
    vault_id TEXT NOT NULL,
    owner TEXT NOT NULL,
    amount INTEGER NOT NULL
);
CREATE TABLE test_trades (
    id INTEGER PRIMARY KEY,
    order_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    FOREIGN KEY (order_id) REFERENCES test_orders(id)
);
"#;

    const TEST_DATA_SQL: &str = r#"
INSERT INTO test_orders (vault_id, owner, amount) VALUES
    ('vault1', 'owner1', 100),
    ('vault2', 'owner2', 200);
INSERT INTO test_trades (order_id, amount) VALUES
    (1, 50),
    (2, 75);
"#;

    const TEST_DATA_SQL_2: &str = r#"
INSERT INTO test_orders (vault_id, owner, amount) VALUES 
    ('vault3', 'owner3', 300);
INSERT INTO test_trades (order_id, amount) VALUES 
    (3, 125);
"#;

    fn create_test_files(temp_dir: &TempDir) -> (String, String) {
        let schema_path = temp_dir.path().join("schema.sql");
        let data_path = temp_dir.path().join("data.sql");

        fs::write(&schema_path, TEST_TABLE_SCHEMA).unwrap();
        fs::write(&data_path, TEST_DATA_SQL).unwrap();

        (
            schema_path.to_string_lossy().to_string(),
            data_path.to_string_lossy().to_string(),
        )
    }

    #[tokio::test]
    async fn test_default_paths() {
        let temp_dir = TempDir::new().unwrap();
        let (schema_path, data_path) = create_test_files(&temp_dir);

        let dump = DbDump {
            data_sql: vec![data_path],
            table_schema_file: schema_path,
            end_block: 12345,
            db_path: None,
            dump_file_path: None,
        };

        let result = dump.execute().await;
        assert!(result.is_ok());

        assert!(Path::new("src/commands/local_db/local_db_12345.db").exists());
        assert!(Path::new("src/commands/local_db/local_db_12345.sql").exists());
        assert!(Path::new("src/commands/local_db/local_db_12345.sql.gz").exists());

        let dump_contents = fs::read_to_string("src/commands/local_db/local_db_12345.sql").unwrap();
        assert!(dump_contents.contains("CREATE VIEW vault_deltas"));

        let _ = fs::remove_file("src/commands/local_db/local_db_12345.db");
        let _ = fs::remove_file("src/commands/local_db/local_db_12345.sql");
        let _ = fs::remove_file("src/commands/local_db/local_db_12345.sql.gz");
    }

    #[tokio::test]
    async fn test_custom_paths() {
        let temp_dir = TempDir::new().unwrap();
        let (schema_path, data_path) = create_test_files(&temp_dir);

        let custom_db_path = temp_dir.path().join("custom_test.db");
        let custom_dump_path = temp_dir.path().join("custom_dump.sql");

        let dump = DbDump {
            data_sql: vec![data_path],
            table_schema_file: schema_path,
            end_block: 12345,
            db_path: Some(custom_db_path.to_string_lossy().to_string()),
            dump_file_path: Some(custom_dump_path.to_string_lossy().to_string()),
        };

        let result = dump.execute().await;
        assert!(result.is_ok());

        assert!(custom_db_path.exists());
        assert!(temp_dir.path().join("custom_dump.sql").exists());
        assert!(temp_dir.path().join("custom_dump.sql.gz").exists());
    }

    #[tokio::test]
    async fn test_end_to_end_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let (schema_path, data_path) = create_test_files(&temp_dir);

        let db_path = temp_dir.path().join("test_workflow.db");
        let dump_path = temp_dir.path().join("test_dump.sql");

        let dump = DbDump {
            data_sql: vec![data_path],
            table_schema_file: schema_path,
            end_block: 99999,
            db_path: Some(db_path.to_string_lossy().to_string()),
            dump_file_path: Some(dump_path.to_string_lossy().to_string()),
        };

        let result = dump.execute().await;
        assert!(result.is_ok());

        assert!(db_path.exists());

        let gz_path = temp_dir.path().join("test_dump.sql.gz");
        assert!(gz_path.exists());
        assert!(temp_dir.path().join("test_dump.sql").exists());

        let output = std::process::Command::new("gunzip")
            .arg("-c")
            .arg(&gz_path)
            .output()
            .unwrap();

        let dump_content = String::from_utf8(output.stdout).unwrap();
        assert!(dump_content.contains("test_orders"));
        assert!(dump_content.contains("test_trades"));
        assert!(dump_content.contains("vault1"));
        assert!(dump_content.contains("owner1"));
    }

    #[tokio::test]
    async fn test_database_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let (schema_path, data_path) = create_test_files(&temp_dir);

        let db_path = temp_dir.path().join("cleanup_test.db");
        let dump_path = temp_dir.path().join("cleanup_dump.sql");

        fs::write(&db_path, "dummy content").unwrap();
        assert!(db_path.exists());

        let dump = DbDump {
            data_sql: vec![data_path],
            table_schema_file: schema_path,
            end_block: 77777,
            db_path: Some(db_path.to_string_lossy().to_string()),
            dump_file_path: Some(dump_path.to_string_lossy().to_string()),
        };

        let result = dump.execute().await;
        assert!(result.is_ok());

        assert!(db_path.exists());
        let db_content = fs::read(&db_path).unwrap();
        assert!(db_content.starts_with(b"SQLite format 3"));
    }

    #[tokio::test]
    async fn test_multi_file_data_sql_applied_in_order() {
        let temp_dir = TempDir::new().unwrap();

        // Create schema and two data files
        let schema_path = temp_dir.path().join("schema.sql");
        let data_path_1 = temp_dir.path().join("data1.sql");
        let data_path_2 = temp_dir.path().join("data2.sql");

        fs::write(&schema_path, TEST_TABLE_SCHEMA).unwrap();
        fs::write(&data_path_1, TEST_DATA_SQL).unwrap();
        fs::write(&data_path_2, TEST_DATA_SQL_2).unwrap();

        let db_path = temp_dir.path().join("multi.db");
        let dump_path = temp_dir.path().join("multi_dump.sql");

        let dump = DbDump {
            data_sql: vec![
                data_path_1.to_string_lossy().to_string(),
                data_path_2.to_string_lossy().to_string(),
            ],
            table_schema_file: schema_path.to_string_lossy().to_string(),
            end_block: 42,
            db_path: Some(db_path.to_string_lossy().to_string()),
            dump_file_path: Some(dump_path.to_string_lossy().to_string()),
        };

        let result = dump.execute().await;
        assert!(result.is_ok());

        let gz_path = temp_dir.path().join("multi_dump.sql.gz");
        assert!(gz_path.exists());
        assert!(temp_dir.path().join("multi_dump.sql").exists());

        let output = std::process::Command::new("gunzip")
            .arg("-c")
            .arg(&gz_path)
            .output()
            .unwrap();

        let dump_content = String::from_utf8(output.stdout).unwrap();
        // From first file
        assert!(dump_content.contains("vault1"));
        assert!(dump_content.contains("owner1"));
        // From second file
        assert!(dump_content.contains("vault3"));
        assert!(dump_content.contains("owner3"));
    }

    #[tokio::test]
    async fn test_schema_only_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("schema.sql");
        fs::write(&schema_path, TEST_TABLE_SCHEMA).unwrap();
        let db_path = temp_dir.path().join("schema_only.db");
        let dump_path = temp_dir.path().join("schema_only.sql");

        let dump = DbDump {
            data_sql: vec![],
            table_schema_file: schema_path.to_string_lossy().to_string(),
            end_block: 13,
            db_path: Some(db_path.to_string_lossy().to_string()),
            dump_file_path: Some(dump_path.to_string_lossy().to_string()),
        };

        let result = dump.execute().await;
        assert!(result.is_ok());

        let gz_path = temp_dir.path().join("schema_only.sql.gz");
        assert!(gz_path.exists());
        assert!(temp_dir.path().join("schema_only.sql").exists());
    }
}
