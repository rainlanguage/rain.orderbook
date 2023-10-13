use ethers::prelude::*;
use std::{env, fs};

pub fn _abigen_rust_generation() -> anyhow::Result<()> {
    process_abi_json_to_rust("tests/generated");

    Ok(())
}

/// Use the given path (relative to the current directory) to search ABI JSON
/// files and generate a definition in Rust using abigen.
///
/// The output path will be the same that the provided.
fn process_abi_json_to_rust(dir_path: &str) {
    let current_directory = env::current_dir().expect("cannot get current directory");
    let output_directory = current_directory.join(dir_path.to_owned() + "/abigen");

    if !output_directory.is_dir() {
        if let Err(err) = fs::create_dir_all(&output_directory) {
            eprintln!("Error creating directory: {}", err);
        }
    }

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    // Only reading ABI JSON files
                    if extension == "json" {
                        // The source path of this ABI
                        let abi_source_path = path.to_str().unwrap();

                        // The ABI name based on the path without the extension
                        let abi_name = path
                            .file_stem()
                            .expect("cannot get filename")
                            .to_str()
                            .unwrap();

                        // The out path where the .rs file will be write
                        let out_file = current_directory
                            // .join(abi_source_path.clone().replace(".json", ".rs"));
                            .join(dir_path.to_owned() + "/abigen/" + abi_name + ".rs");

                        // This allow to remove old files
                        if out_file.exists() {
                            fs::remove_file(&out_file).expect("cannot delete file");
                        }

                        Abigen::new(abi_name, abi_source_path)
                            .expect("failed on new call")
                            .generate()
                            .expect("failed on generate")
                            .write_to_file(out_file)
                            .expect("failed on write");
                    }
                }
            }
        }
    }
}
