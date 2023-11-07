// extern crate url;
mod deploy;
mod utils;
use clap::{Parser, Subcommand};

use colored::*;
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
            let resp = run_cmd("npm", &["install"]);

            if !resp {
                eprintln!("{}", "Error: Failed at npm install".red());
                std::process::exit(1);
            }

            Ok(())
        }

        Subgraph::Build => {
            let resp = run_cmd("npm", &["run", "codegen"]);
            if !resp {
                eprintln!("{}", "Error: Failed at npm run codegen".red());
                std::process::exit(1);
            }

            let resp = run_cmd("npm", &["run", "build"]);
            if !resp {
                eprintln!("{}", "Error: Failed at npm run build".red());
                std::process::exit(1);
            }

            Ok(())
        }

        Subgraph::Test => {
            let resp = run_cmd("nix", &["run", ".#ci-test"]);
            if !resp {
                std::process::exit(1);
            }

            Ok(())
        }

        Subgraph::Deploy(args) => {
            match deploy_subgraph(args) {
                Ok(_) => {
                    return Ok(());
                }
                Err(err) => {
                    // Error occurred, print the error message and exit
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}
