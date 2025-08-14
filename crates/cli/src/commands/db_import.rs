use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
pub struct DbImport;

impl DbImport {
    pub async fn execute(self) -> Result<()> {
        let db_path = "events.db";
        let sql_file_path = "events.sql";
        let dump_file_path = "events_dump.sql";
        let compressed_file_path = "events_dump.sql.gz";

        // Check if SQL file exists
        if !Path::new(sql_file_path).exists() {
            anyhow::bail!("SQL file not found: {}", sql_file_path);
        }

        // Remove existing database if it exists
        if Path::new(db_path).exists() {
            fs::remove_file(db_path)?;
            println!("Removed existing database: {}", db_path);
        }

        // Create database and tables
        println!("Creating SQLite database: {}", db_path);
        let tables_sql_path = "src/tables-v2.sql";

        // Create tables first
        let output = Command::new("sqlite3")
            .arg(db_path)
            .arg(format!(".read {}", tables_sql_path))
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to create tables: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        println!("Created database tables");

        // Execute the events SQL file
        println!("Importing data from: {}", sql_file_path);
        let output = Command::new("sqlite3")
            .arg(db_path)
            .arg(format!(".read {}", sql_file_path))
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to import SQL file: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        println!("Successfully imported data");

        // Dump the database
        println!("Dumping database to: {}", dump_file_path);
        let output = Command::new("sqlite3").arg(db_path).arg(".dump").output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to dump database: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        fs::write(dump_file_path, output.stdout)?;
        let dump_size = fs::metadata(dump_file_path)?.len();
        println!(
            "Dump file size: {} bytes ({:.2} MB)",
            dump_size,
            dump_size as f64 / 1_000_000.0
        );

        // Compress the dump
        println!("Compressing dump to: {}", compressed_file_path);
        let output = Command::new("gzip")
            .arg("-f") // force overwrite
            .arg(dump_file_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to compress dump: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let compressed_size = fs::metadata(compressed_file_path)?.len();
        let compression_ratio = compressed_size as f64 / dump_size as f64;

        println!(
            "Compressed file size: {} bytes ({:.2} MB)",
            compressed_size,
            compressed_size as f64 / 1_000_000.0
        );
        println!(
            "Compression ratio: {:.2}% ({:.1}x smaller)",
            compression_ratio * 100.0,
            1.0 / compression_ratio
        );

        // Show database info
        let db_size = fs::metadata(db_path)?.len();
        println!(
            "Original database size: {} bytes ({:.2} MB)",
            db_size,
            db_size as f64 / 1_000_000.0
        );

        // Get row counts
        let output = Command::new("sqlite3")
            .arg(db_path)
            .arg("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")
            .output()?;

        if output.status.success() {
            let tables = String::from_utf8_lossy(&output.stdout);
            println!("\nTable row counts:");
            for table in tables.lines() {
                if !table.is_empty() {
                    let count_output = Command::new("sqlite3")
                        .arg(db_path)
                        .arg(format!("SELECT COUNT(*) FROM {};", table))
                        .output()?;

                    if count_output.status.success() {
                        let count = String::from_utf8_lossy(&count_output.stdout)
                            .trim()
                            .to_string();
                        println!("  {}: {} rows", table, count);
                    }
                }
            }
        }

        Ok(())
    }
}
