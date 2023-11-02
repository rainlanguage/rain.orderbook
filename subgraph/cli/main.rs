mod deploy;
mod utils;
use clap::{Args, Parser, Subcommand};

use deploy::{deploy, Config};
use utils::run_cmd;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub subgraph: Subgraph,
}

#[derive(Subcommand)]
pub enum Subgraph {
    #[command(about = "Install dependecies for the rain subgraph")]
    Install,
    #[command(about = "Build the rain subgraph")]
    Build,
    #[command(about = "Test the rain subgraph")]
    Test,
    #[command(about = "Deploy the rain subgraph")]
    Deploy(DeployCommand),
}

#[derive(Args, Debug)]
pub struct DeployCommand {
    /// Endpoint URL where the subgraph will be deployed
    url: String,
    /// Subgraph token to deploy the subgraph
    key: String,
    /// Network that the subgraph will index
    network: String,
    /// Block number where the subgraph will start indexing
    block_number: String,
    /// Contract address that the subgraph will be indexing (Assuming one address)
    address: String,
}

fn main() {
    let args = Cli::parse();

    match args.subgraph {
        Subgraph::Install => {
            run_cmd("npm", &["install"]);
        }

        Subgraph::Build => {
            // Use a arbitrary address to put the endpoint up
            let config = Config {
                contract_address: &"0x0000000000000000000000000000000000000000".to_string(),
                block_number: 0,
            };

            let _ = deploy(config);

            // Get the schema from the endpoint
            let write_schema = run_cmd(
                "graphql-client",
                &[
                    "introspect-schema",
                    "--output",
                    "tests/subgraph/query/schema.json",
                    "http://localhost:8000/subgraphs/name/test/test",
                ],
            );

            if write_schema {
                println!("HERE_1: schema was wrote");
            } else {
                println!("HERE_2: failed to write schemag");
            }

            ()
        }

        Subgraph::Test => {
            println!("Hello tests 🧪");
            todo!("Test CI does not implemented");
        }
        Subgraph::Deploy(args) => {
            println!("🚀 Hello, deploy with: {:?}", args);
            todo!("Deploy CI does not implemented");
        }
    }
}
