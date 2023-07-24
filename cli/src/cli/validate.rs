use crate::meta::KnownMeta;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Validate {
    /// The known meta to validate against.
    #[arg(short, long)]
    meta: KnownMeta,
    /// The input path to the json serialized metadata to validate against the
    /// known schema.
    #[arg(short, long)]
    input_path: PathBuf,
}

pub fn validate(v: Validate) -> anyhow::Result<()> {
    let data: Vec<u8> = std::fs::read(v.input_path)?;
    // If we can normalize the input data then it is valid.
    let _normalized = v.meta.normalize(&data)?;
    Ok(())
}