use anyhow::anyhow;
use mustache::MapBuilder;
use std::fs;
// use std::{fs, process::Command};

use crate::utils::run_cmd;

pub struct Config<'a> {
    // contracts address
    pub contract_address: &'a String,
    // block-number
    pub block_number: u64,
}

pub fn deploy(config: Config) -> anyhow::Result<bool> {
    let subgraph_template = "subgraph.template.yaml";
    let output_path = "subgraph.yaml";
    // let root_dir = "./";
    let end_point = "http://localhost:8020/";
    let subgraph_name = "test/test";

    let data = MapBuilder::new()
        .insert_str("network", "localhost")
        .insert_str("orderbook", config.contract_address)
        .insert_str("blockNumber", config.block_number.to_string())
        .build();

    let template = fs::read_to_string(subgraph_template)?;
    let renderd = mustache::compile_str(&template)?.render_data_to_string(&data)?;
    let _ = fs::write(output_path, renderd)?;

    // Generate the subgraph code
    let is_built = run_cmd("bash", &["-c", "npx graph codegen && npx graph build"]);
    if !is_built {
        return Err(anyhow!("Failed to build subgraph"));
    }

    // Create the endpoint node
    let is_node_up = run_cmd(
        "bash",
        &[
            "-c",
            &format!("npx graph create --node {} {}", end_point, subgraph_name),
        ],
    );
    if !is_node_up {
        return Err(anyhow!("Failed to create subgraph endpoint node"));
    }

    // Deploy Subgraph to the endpoint
    let is_deploy = run_cmd(
        "bash",
        &[
            "-c",
            &format!(
                "npx graph deploy --node {} --ipfs http://localhost:5001 {}  --version-label 1",
                end_point, subgraph_name
            ),
        ],
    );
    if !is_deploy {
        return Err(anyhow!("Failed to deploy subgraph"));
    }

    Ok(true)
}
