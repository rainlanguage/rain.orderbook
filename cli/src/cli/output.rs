use std::io::Write;
use std::path::PathBuf;
use strum::EnumString;
use strum::EnumIter;

#[derive(serde::Serialize, Clone, Copy, EnumString, EnumIter, strum::Display)]
#[strum(serialize_all = "kebab_case")]
#[serde(rename_all = "kebab-case")]
#[repr(u64)]
pub enum SupportedOutputEncoding {
    Binary,
    Hex,
}

pub fn output(output_path: &Option<PathBuf>, output_encoding: SupportedOutputEncoding, bytes: &[u8]) -> anyhow::Result<()> {
    let hex_encoded: String;
    let encoded_bytes: &[u8] = match output_encoding {
        SupportedOutputEncoding::Binary => bytes,
        SupportedOutputEncoding::Hex => {
            hex_encoded = format!("0x{}", hex::encode(bytes));
            hex_encoded.as_bytes()
        },
    };
    Ok(if let Some(output_path) = output_path {
        std::fs::write(output_path, encoded_bytes)?
    } else {
        std::io::stdout().write_all(encoded_bytes)?
    })
}