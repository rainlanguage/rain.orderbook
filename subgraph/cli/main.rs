// extern crate url;
mod deploy;
mod utils;
use clap::{Parser, Subcommand};

// use colored::*;
use deploy::{deploy_subgraph, DeployArgs};
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
    Deploy(DeployArgs),
}

fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    match args.subgraph {
        Subgraph::Install => {
            run_cmd("npm", &["install"]);

            Ok(())
        }

        Subgraph::Build => {
            run_cmd("npm", &["run", "codegen"]);
            run_cmd("npm", &["run", "build"]);

            Ok(())
        }

        Subgraph::Test => {
            run_cmd("nix", &["run", ".#ci-test"]);

            Ok(())
        }
        Subgraph::Deploy(args) => {
            println!("\n🚀 Hello deploy");
            let _ = deploy_subgraph(args);

            // if args.url.scheme() != "http" && args.url.scheme() != "https" {
            //     eprintln!("Error: Invalid URL provided");
            //     std::process::exit(1);
            // }

            Ok(())

        }
    }
}
