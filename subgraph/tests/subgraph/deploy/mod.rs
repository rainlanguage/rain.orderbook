use anyhow::anyhow;
use mustache::MapBuilder;
use std::{fs, process::Command};

pub struct Config<'a> {
    // contracts address
    pub contract_address: &'a String,
    // block-number
    pub block_number: u64,
}

pub fn deploy(config: Config) -> anyhow::Result<bool> {
    let subgraph_template = "subgraph.template.yaml";
    let output_path = "subgraph.yaml";
    let root_dir = "./";
    let end_point = "http://localhost:8020/";
    let subgraph_name = "test/test";

    let data = MapBuilder::new()
        .insert_str("network", "localhost")
        .insert_str("orderbook", config.contract_address)
        .insert_str("blockNumber", config.block_number.to_string())
        .build();

    let template = fs::read_to_string(subgraph_template)
        .expect(&format!("Fail to read {}", subgraph_template));

    let renderd = mustache::compile_str(&template)
        .expect("Failed to compile template")
        .render_data_to_string(&data)
        .expect("Failed to render template");

    let _write = fs::write(output_path, renderd)?;

    let output = Command::new("bash")
        .current_dir(format!(
            "{}/{}",
            std::env::current_dir().unwrap().display(),
            root_dir
        ))
        .args(&["-c", "npx graph codegen && npx graph build"])
        .output()
        .expect("Failed graph codegen and graph build command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("{}", stderr));
    }

    let _output = Command::new("bash")
        .current_dir(format!(
            "{}/{}",
            std::env::current_dir().unwrap().display(),
            root_dir
        ))
        .args(&[
            "-c",
            &format!("npx graph create --node {} {}", end_point, subgraph_name),
        ])
        .output()
        .expect("Failed graph create command");

    let output = Command::new("bash")
        .current_dir(format!(
            "{}/{}",
            std::env::current_dir().unwrap().display(),
            root_dir
        ))
        .args(&[
            "-c",
            &format!(
                "npx graph deploy --node {} --ipfs http://localhost:5001 {}  --version-label 1",
                end_point, subgraph_name
            ),
        ])
        .output()
        .expect("Failed local deploy command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("{}", stderr));
    }

    Ok(true)
}
