use anyhow::Result;
use clap::Parser;
use std::fs;
use std::process::Command;

#[derive(Parser)]
pub struct DbDump {
    #[clap(long)]
    pub input_file: String,
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
        let sql_file_path = &self.input_file;
        let db_path = self
            .db_path
            .unwrap_or_else(|| format!("src/commands/local_db/local_db_{}.db", self.end_block));
        let dump_file_path = self
            .dump_file_path
            .unwrap_or_else(|| format!("src/commands/local_db/local_db_{}.sql", self.end_block));

        let _ = fs::remove_file(&db_path);

        let tables_sql_path = &self.table_schema_file;

        let _ = Command::new("sqlite3")
            .arg(&db_path)
            .arg(format!(".read {}", tables_sql_path))
            .status()?;

        let _ = Command::new("sqlite3")
            .arg(&db_path)
            .arg(format!(".read {}", sql_file_path))
            .status()?;

        let output = Command::new("sqlite3")
            .arg(&db_path)
            .arg(".dump")
            .output()?;

        fs::write(&dump_file_path, output.stdout)?;

        Command::new("gzip")
            .arg("-f")
            .arg(&dump_file_path)
            .status()?;

        Ok(())
    }
}
