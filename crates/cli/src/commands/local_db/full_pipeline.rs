use super::{DbImport, DecodeEvents, EventsToSql, FetchEvents};
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct FullPipeline {
    #[clap(
        short,
        long,
        help = "Custom output directory for all generated files (optional)"
    )]
    pub output_dir: Option<String>,
}

impl FullPipeline {
    pub async fn execute(self) -> Result<()> {
        println!("Starting full pipeline execution...");
        println!("=================================");

        // Determine the base directory for all files
        let base_dir = self.output_dir.as_deref().unwrap_or("src/commands/local_db");
        
        // Step 1: Fetch events
        println!("\n1. Fetching events...");
        let fetch_events = FetchEvents {
            output_file: None, // Let it use default naming
        };
        fetch_events.execute().await?;

        // Find the events file - check both current directory and target directory
        let events_file = find_latest_events_file()
            .or_else(|_| find_latest_events_file_in_dir(base_dir))?;
        
        // Move the file to the target directory if needed
        let target_events_file = if events_file.starts_with(base_dir) {
            // File is already in the target directory
            events_file
        } else if let Some(ref dir) = self.output_dir {
            let filename = std::path::Path::new(&events_file)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let target_path = format!("{}/{}", dir, filename);
            std::fs::create_dir_all(dir)?;
            std::fs::rename(&events_file, &target_path)?;
            target_path
        } else {
            // Move to src/commands/local_db/
            let filename = std::path::Path::new(&events_file)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let target_path = format!("src/commands/local_db/{}", filename);
            std::fs::rename(&events_file, &target_path)?;
            target_path
        };

        // Step 2: Decode events
        println!("\n2. Decoding events...");
        let decode_events = DecodeEvents {
            input_file: target_events_file.clone(),
            output_file: None, // Let it use default naming based on input
        };
        decode_events.execute().await?;

        // Find the decoded file that was just created and move it to target directory
        let block_num = extract_block_from_events_file(&target_events_file);
        let decoded_filename = if let Some(block) = &block_num {
            format!("decoded_events_{}.json", block)
        } else {
            "decoded_events.json".to_string()
        };
        
        // The decoded file is created in the current directory, move it to target
        let target_decoded_file = format!("{}/{}", base_dir, decoded_filename);
        if std::path::Path::new(&decoded_filename).exists() {
            std::fs::rename(&decoded_filename, &target_decoded_file)?;
        }
        
        let decoded_file = target_decoded_file;

        // Step 3: Convert to SQL
        println!("\n3. Converting to SQL...");
        let events_to_sql = EventsToSql {
            input: std::path::PathBuf::from(&decoded_file),
            output: None, // Let it use default naming based on input
        };
        events_to_sql.execute().await?;

        // Find the SQL file that was just created and move it to target directory
        let sql_filename = if let Some(block) = &block_num {
            format!("events_{}.sql", block)
        } else {
            "events.sql".to_string()
        };
        
        // The SQL file is created in the current directory, move it to target
        let target_sql_file = format!("{}/{}", base_dir, sql_filename);
        if std::path::Path::new(&sql_filename).exists() {
            std::fs::rename(&sql_filename, &target_sql_file)?;
        }
        
        let sql_file = target_sql_file;

        // Step 4: Import to database and create dump
        println!("\n4. Importing to database and creating dump...");
        let db_import = DbImport {
            input: sql_file.clone(),
        };
        db_import.execute().await?;

        println!("\n=================================");
        println!("✅ Full pipeline completed successfully!");

        let block_num = extract_block_from_events_file(&target_events_file);
        if let Some(block) = block_num {
            println!("📁 Generated files for block {}:", block);
            println!("   • {}/events_{}.json", base_dir, block);
            println!("   • {}/decoded_events_{}.json", base_dir, block);
            println!("   • {}/events_{}.sql", base_dir, block);
            println!("   • {}/events_{}.db", base_dir, block);
            println!("   • {}/dump_{}.sql", base_dir, block);
            println!("   • {}/dump_{}.sql.gz", base_dir, block);
        }

        Ok(())
    }
}

fn find_latest_events_file() -> Result<String> {
    use std::fs;
    use std::time::SystemTime;

    let current_dir = std::env::current_dir()?;
    let mut latest_file = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with("events_") && filename.ends_with(".json") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > latest_time {
                            latest_time = modified;
                            latest_file = Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    latest_file.ok_or_else(|| anyhow::anyhow!("No events_*.json files found in current directory"))
}

fn find_latest_events_file_in_dir(dir: &str) -> Result<String> {
    use std::fs;
    use std::time::SystemTime;

    let search_dir = std::path::Path::new(dir);
    if !search_dir.exists() {
        return Err(anyhow::anyhow!("Directory does not exist: {}", dir));
    }

    let mut latest_file = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

    for entry in fs::read_dir(search_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with("events_") && filename.ends_with(".json") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > latest_time {
                            latest_time = modified;
                            latest_file = Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    latest_file.ok_or_else(|| anyhow::anyhow!("No events_*.json files found in directory: {}", dir))
}

fn extract_block_from_events_file(filename: &str) -> Option<String> {
    std::path::Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
            name.strip_prefix("events_")
                .and_then(|s| s.strip_suffix(".json"))
                .map(|s| s.to_string())
        })
}
