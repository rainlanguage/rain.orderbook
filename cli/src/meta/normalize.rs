use super::KnownMeta;
use super::interpreter_caller::v1::InterpreterCallerMeta;
use super::op::v1::OpMeta;
use super::solidity_abi::v2::SolidityAbi;

fn normalize_json<'de, T: serde::Deserialize<'de> + serde::Serialize + validator::Validate>(data: &'de [u8]) -> anyhow::Result<Vec<u8>> {
    let parsed = serde_json::from_str::<T>(std::str::from_utf8(data)?)?;
    parsed.validate()?;
    Ok(serde_json::to_string(&parsed)?.as_bytes().to_vec())
}

impl KnownMeta {
    pub fn normalize(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        Ok(match self {
            KnownMeta::SolidityAbiV2 => normalize_json::<SolidityAbi>(data)?,
            KnownMeta::InterpreterCallerMetaV1 => normalize_json::<InterpreterCallerMeta>(data)?,
            KnownMeta::OpV1 => normalize_json::<OpMeta>(data)?,
        })
    }
}