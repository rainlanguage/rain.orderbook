use anyhow::Result;
use clap::command;
use clap::{Parser, Subcommand};

pub mod build;
pub mod schema;
pub mod validate;
pub mod magic;
pub mod solc;
pub mod output;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    meta: Meta,
}

#[derive(Subcommand)]
pub enum Meta {
    #[command(subcommand)]
    Schema(schema::Schema),
    Validate(validate::Validate),
    #[command(subcommand)]
    Magic(magic::Magic),
    Build(build::Build),
    #[command(subcommand)]
    Solc(solc::Solc),
}

pub fn dispatch(meta: Meta) -> Result<()> {
    match meta {
        Meta::Schema(schema) => schema::dispatch(schema),
        Meta::Validate(validate) => validate::validate(validate),
        Meta::Magic(magic) => magic::dispatch(magic),
        Meta::Build(build) => build::build(build),
        Meta::Solc(solc) => solc::dispatch(solc),
    }
}

pub fn main() -> Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;

    let cli = Cli::parse();
    dispatch(cli.meta)
}
