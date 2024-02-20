use std::collections::HashMap;
use std::sync::{MutexGuard, PoisonError};

use crate::abi_error::AbiDecodedErrorType;
use crate::forked_evm_cache::{ForkCallError, FORKED_EVM_CACHE};
use crate::frontmatter::{try_parse_frontmatter, FrontmatterError};
use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
use alloy_primitives::{bytes::Bytes, Address, FixedBytes};
use alloy_sol_types::SolCall;
use forker::ForkedEvm;
use rain_interpreter_bindings::DeployerISP::iParserCall;
use rain_interpreter_bindings::IExpressionDeployerV3::deployExpression2Call;
use rain_interpreter_bindings::IParserV1::parseCall;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForkParseError {
    #[error("Fork Cache Poisoned")]
    ForkCachePoisoned,
    #[error(transparent)]
    ForkCallFailed(#[from] ForkCallError),
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

impl<'a> From<PoisonError<MutexGuard<'a, HashMap<String, ForkedEvm>>>> for ForkParseError {
    fn from(_value: PoisonError<MutexGuard<'a, HashMap<String, ForkedEvm>>>) -> Self {
        Self::ForkCachePoisoned
    }
}

/// Arbitrary address used to call from
const SENDER_ADDRESS: Address = Address::repeat_byte(0x1);

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
    let cache_key = FORKED_EVM_CACHE
        .get_or_create(rpc_url, block_number_val)
        .await?;

    // Call deployer contract: iParserCall
    let calldata = iParserCall {}.abi_encode();
    let response = FORKED_EVM_CACHE
        .call(cache_key.clone(), SENDER_ADDRESS, deployer, &calldata)
        .await?;
    let parser_address = Address::from_word(
        FixedBytes::try_from(response.as_ref())
            .map_err(|_| ForkParseError::ReadParserAddressFailed)?,
    );

    // Call parser contract: parseCall
    let calldata = parseCall {
        data: rainlang.as_bytes().to_vec(),
    }
    .abi_encode();
    let expression_config = FORKED_EVM_CACHE
        .call(cache_key.clone(), SENDER_ADDRESS, parser_address, &calldata)
        .await?;

    // Call deployer: deployExpression2Call
    let mut calldata = deployExpression2Call::SELECTOR.to_vec();
    calldata.extend_from_slice(&expression_config);
    FORKED_EVM_CACHE
        .call(cache_key, SENDER_ADDRESS, deployer, &calldata)
        .await?;
    let expression_config_bytes = expression_config;

    Ok(expression_config_bytes)
}
