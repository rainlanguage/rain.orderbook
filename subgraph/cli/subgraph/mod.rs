use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
struct SubgraphTemplate {
    #[serde(rename = "specVersion")]
    spec_version: String,
    schema: Schema,
    #[serde(rename = "dataSources")]
    data_sources: Vec<DataSource>,
    templates: Vec<DataSource>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Schema {
    file: String,
}

/// Struct definition for DataSource and Template fields in subgraph YAML file
/// that describe every field for generated code
#[derive(Debug, Serialize, Deserialize)]
struct DataSource {
    kind: String,
    name: String,
    network: String,
    source: Source,
    mapping: Mapping,
}

#[derive(Debug, Serialize, Deserialize)]
struct Mapping {
    kind: String,
    #[serde(rename = "apiVersion")]
    api_version: String,
    language: String,
    entities: Vec<String>,
    abis: Vec<Abi>,
    #[serde(rename = "eventHandlers")]
    event_handlers: Vec<EventHandler>,
    file: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Abi {
    name: String,
    file: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventHandler {
    event: String,
    handler: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    abi: String,
    #[serde(rename = "startBlock", skip_serializing_if = "Option::is_none")]
    start_block: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Template {
    network: String,
    source: Source,
}

/// Arguments for building the yaml file to generate the code used by subgraph
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Network that the subgraph will index
    #[arg(long)]
    pub network: String,

    /// Block number where the subgraph will start indexing
    #[arg(long = "block")]
    pub block_number: u64,

    /// Contract address that the subgraph will be indexing (Assuming one address)
    #[arg(long)]
    pub address: String,
}

pub fn build(args: BuildArgs) -> anyhow::Result<()> {
    let mut file = File::open("subgraph.template.yaml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut yaml_data: SubgraphTemplate = serde_yaml::from_str(&contents)?;
    // Update values in dataSources using the given arguments
    for data_source in &mut yaml_data.data_sources {
        data_source.network = args.network.clone();
        data_source.source.address = Some(format!("\"{}\"", args.address));
        data_source.source.start_block = Some(args.block_number);
    }

    // Update values in templates using the given arguments
    for template in &mut yaml_data.templates {
        template.network = args.network.clone();
    }

    let mut modified_yaml = serde_yaml::to_string(&yaml_data)?;

    // TODO: Modifiy this since when serializing the string does not add the quotes.
    // And when the quotes are added using format! macro, then two or three quotes
    // are added.
    modified_yaml = modified_yaml.replace("'\"", "'");
    modified_yaml = modified_yaml.replace("\"'", "'");

    let mut modified_file = File::create("subgraph.yaml")?;

    modified_file.write_all(modified_yaml.as_bytes())?;

    Ok(())
}
