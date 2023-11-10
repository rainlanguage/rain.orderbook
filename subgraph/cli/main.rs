mod cmd;
mod subgraph;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub subgraph: Subgraph,
}

#[derive(Subcommand)]
pub enum Subgraph {
    #[command(about = "Build the rain subgraph code")]
    Build,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.subgraph {
        Subgraph::Build => {
            let config = subgraph::BuildArgs {
                address: "0xff000000000000000000000000000000000000ff".to_string(),
                network: "localhost".to_string(),
                block_number: 0,
            };

            let resp_build = subgraph::build(config);
            if resp_build.is_err() {
                std::process::exit(1);
            }

            let resp_codegen = cmd::run("npm", &["run", "codegen"]);
            if resp_codegen.is_err() {
                std::process::exit(1);
            }

            let resp_build = cmd::run("npm", &["run", "build"]);
            if resp_build.is_err() {
                std::process::exit(1);
            }

            Ok(())
        }
    }
}
