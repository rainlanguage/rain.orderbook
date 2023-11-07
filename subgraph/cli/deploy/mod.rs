use anyhow::anyhow;
use mustache::MapBuilder;
use std::fs;
// use std::{fs, process::Command};

use crate::utils::run_cmd;

use clap::Args;
use url::Url;

#[derive(Args, Debug)]
pub struct DeployArgs {
    /// Subgraph name (eg: User/SubgraphName)
    #[arg(long = "name")]
    pub subgraph_name: String,

    /// Endpoint URL where the subgraph will be deployed
    #[arg(long)]
    pub url: Url,

    /// Network that the subgraph will index
    #[arg(long)]
    pub network: String,

    /// Block number where the subgraph will start indexing
    #[arg(long = "block")]
    pub block_number: u32,

    /// Contract address that the subgraph will be indexing (Assuming one address)
    #[arg(long)]
    pub address: String,

    /// (Optional) Subgraph token to deploy the subgraph
    #[arg(long)]
    pub key: Option<String>,
}

pub fn deploy_subgraph(config: DeployArgs) -> anyhow::Result<()> {
    if config.url.scheme() != "http" && config.url.scheme() != "https" {
        return Err(anyhow!("Invalid URL provided"));
    }

    let subgraph_template = "subgraph.template.yaml";
    let output_path = "subgraph.yaml";

    let end_point = config.url.as_str();
    let subgraph_name = config.subgraph_name;

    let data = MapBuilder::new()
        .insert_str("network", config.network)
        .insert_str("orderbook", config.address)
        .insert_str("blockNumber", config.block_number.to_string())
        .build();

    let template = fs::read_to_string(subgraph_template)?;
    let renderd = mustache::compile_str(&template)?.render_data_to_string(&data)?;
    let _ = fs::write(output_path, renderd)?;

    // Generate the subgraph code
    // let is_built = run_cmd("bash", &["-c", "npx graph codegen && npx graph build"]);
    let is_built = run_cmd("bash", &["-c", "npx graph codegen"]);
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

    if config.url.host_str() == Some("localhost") {
        // Deploy Subgraph to the endpoint local
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
    } else {
        // Deploy Subgraph to the endpoint
        let is_deploy = run_cmd(
            "bash",
            &[
                "-c",
                &format!(
                    "npx graph deploy --node {} {}  --version-label 1",
                    end_point, subgraph_name
                ),
            ],
        );
        if !is_deploy {
            return Err(anyhow!("Failed to deploy subgraph"));
        }
    }

    Ok(())
}
