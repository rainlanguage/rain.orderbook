use crate::commands::local_db::executor::RusqliteExecutor;
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
use rain_orderbook_common::local_db::query::create_tables::create_tables_stmt;
use rain_orderbook_common::local_db::query::create_views::create_views_batch;
use rain_orderbook_common::local_db::query::insert_db_metadata::insert_db_metadata_stmt;
use rain_orderbook_common::local_db::query::{LocalDbQueryExecutor, SqlStatement};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

const FAST_IMPORT_PRAGMAS: &str = concat!(
    "PRAGMA journal_mode=MEMORY;",
    "PRAGMA synchronous=OFF;",
    "PRAGMA temp_store=MEMORY;",
    "PRAGMA locking_mode=EXCLUSIVE;",
    "PRAGMA cache_size=-200000;"
);

/// Import a dump SQL string into a new SQLite DB using the same executor
/// and pragmas as the producer bootstrap.
#[derive(Debug, Parser)]
#[command(about = "Import a local-db dump into a fresh SQLite database")]
pub struct ImportDump {
    #[clap(
        long,
        help = "SQL dump string; if omitted, --dump-file must be provided",
        value_name = "SQL"
    )]
    pub dump: Option<String>,

    #[clap(
        long,
        help = "Path to a file containing the SQL dump; used when --dump is not set",
        value_name = "PATH"
    )]
    pub dump_file: Option<PathBuf>,

    #[clap(
        long,
        help = "Output database path",
        value_name = "PATH",
        default_value = "./imported_dump.db"
    )]
    pub db_path: PathBuf,
}

impl ImportDump {
    pub async fn execute(self) -> Result<()> {
        let dump_sql = match (self.dump, self.dump_file) {
            (Some(sql), None) => sql,
            (None, Some(path)) => fs::read_to_string(&path)?,
            (Some(_), Some(_)) => {
                return Err(anyhow!(
                    "Provide only one of --dump or --dump-file, not both"
                ))
            }
            (None, None) => return Err(anyhow!("Missing dump; pass --dump or --dump-file")),
        };

        if self.db_path.exists() {
            fs::remove_file(&self.db_path)?;
        }

        let exec = RusqliteExecutor::new(&self.db_path);

        // Create tables + db_metadata + views
        let setup_started = Instant::now();
        exec.query_text(&create_tables_stmt())
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        exec.query_text(&insert_db_metadata_stmt(DB_SCHEMA_VERSION))
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        exec.execute_batch(&create_views_batch())
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        let setup_ms = setup_started.elapsed().as_millis();

        // Import dump with fast pragmas
        let mut full_sql = String::with_capacity(FAST_IMPORT_PRAGMAS.len() + dump_sql.len());
        full_sql.push_str(FAST_IMPORT_PRAGMAS);
        full_sql.push_str(&dump_sql);
        let import_stmt = SqlStatement::new(full_sql);
        let import_started = Instant::now();
        exec.query_text(&import_stmt)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        let import_ms = import_started.elapsed().as_millis();

        println!(
            "Imported dump into {} (setup {} ms, import {} ms)",
            self.db_path.display(),
            setup_ms,
            import_ms
        );

        Ok(())
    }
}
