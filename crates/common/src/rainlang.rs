use crate::frontmatter::{try_parse_frontmatter, FrontmatterError};
use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
use alloy_primitives::{bytes::Bytes, Address};
use once_cell::sync::Lazy;
use rain_interpreter_eval::error::AbiDecodedErrorType;
use rain_interpreter_eval::error::ForkCallError;
use rain_interpreter_eval::eval::ForkParseArgs;
use rain_interpreter_eval::fork::Forker;
use rain_interpreter_eval::fork::NewForkedEvm;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

pub static FORKER: Lazy<Arc<Mutex<Forker>>> = Lazy::new(|| Arc::new(Mutex::new(Forker::new())));

#[derive(Debug, Error)]
pub enum ForkParseError {
    #[error("Fork Cache Poisoned")]
    ForkCachePoisoned,
    #[error(transparent)]
    ForkerError(ForkCallError),
    #[error("Fork Call Reverted: {0}")]
    ForkCallReverted(AbiDecodedErrorType),
    #[error("Front Matter: {0}")]
    FrontmatterError(#[from] FrontmatterError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error("Failed to read Parser address from deployer")]
    ReadParserAddressFailed,
}

impl From<AbiDecodedErrorType> for ForkParseError {
    fn from(value: AbiDecodedErrorType) -> Self {
        Self::ForkCallReverted(value)
    }
}
impl From<ForkCallError> for ForkParseError {
    fn from(value: ForkCallError) -> Self {
        match value {
            ForkCallError::AbiDecodedError(v) => Self::ForkCallReverted(v),
            other => Self::ForkerError(other),
        }
    }
}

/// Arbitrary address used to call from
pub const SENDER_ADDRESS: Address = Address::repeat_byte(0x1);

/// checks the front matter validity and parses the given rainlang string
/// with the deployer parsed from the front matter
/// returns abi encoded expression config on Ok variant
pub async fn parse_rainlang_on_fork(
    frontmatter: &str,
    rainlang: &str,
    rpc_url: &str,
    block_number: Option<u64>,
) -> Result<Bytes, ForkParseError> {
    let deployer = try_parse_frontmatter(frontmatter)?.0;

    // Prepare evm fork
    let block_number_val = match block_number {
        Some(b) => b,
        None => {
            let client = ReadableClientHttp::new_from_url(rpc_url.to_string())?;

            client.get_block_number().await?
        }
    };
    let args = NewForkedEvm {
        fork_url: rpc_url.to_owned(),
        fork_block_number: Some(block_number_val),
    };
    let mut forker = FORKER.lock().await;
    forker.add_or_select(args, None).await?;

    let parse_args = ForkParseArgs {
        rainlang_string: rainlang.to_owned(),
        deployer,
    };
    let result = forker.fork_parse(parse_args).await?;

    Ok(result.0.raw.result.0)
}
