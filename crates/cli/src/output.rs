use std::io::Write;
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone)]
pub enum SupportedOutputEncoding {
    Binary,
    Hex,
}

pub fn output(
    output_path: &Option<PathBuf>,
    output_encoding: SupportedOutputEncoding,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let hex_encoded: String;
    let encoded_bytes: &[u8] = match output_encoding {
        SupportedOutputEncoding::Binary => bytes,
        SupportedOutputEncoding::Hex => {
            hex_encoded = alloy::primitives::hex::encode_prefixed(bytes);
            hex_encoded.as_bytes()
        }
    };
    if let Some(output_path) = output_path {
        std::fs::write(output_path, encoded_bytes)?
    } else {
        std::io::stdout().write_all(encoded_bytes)?
    }
    Ok(())
}
