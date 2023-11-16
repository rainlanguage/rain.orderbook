mod cmd;
mod subgraph;

use clap::{Parser, Subcommand};
use tracing::{subscriber, Level};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(long, short = 'd', global = true)]
    pub debug: bool,

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

    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_target(false)
            .init();
    } else {
        subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;
    }

    match args.subgraph {
        Subgraph::Build => {
            let config = subgraph::BuildArgs {
                address: "0xff000000000000000000000000000000000000ff".to_string(),
                network: "localhost".to_string(),
                block_number: 0,
            };

            let resp_build = subgraph::build(config);
            if resp_build.is_err() {
                tracing::error!("{}", resp_build.err().unwrap().to_string());
                std::process::exit(1);
            }

            let resp_codegen_cmd = cmd::run("npm", &["run", "codegen"]);
            if resp_codegen_cmd.is_err() {
                tracing::error!("{}", resp_codegen_cmd.err().unwrap().to_string());
                std::process::exit(1);
            }

            let resp_build_cmd = cmd::run("npm", &["run", "build"]);
            if resp_build_cmd.is_err() {
                tracing::error!("{}", resp_build_cmd.err().unwrap().to_string());
                std::process::exit(1);
            }

            Ok(())
        }
    }
}
