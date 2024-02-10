use crate::error::ForkParseError;
use crate::forked_evm_cache::FORKED_EVM_CACHE;
use crate::front_matter::try_parse_frontmatter;
use alloy_ethers_typecast::transaction::ReadableClientHttp;
use alloy_primitives::{Address, FixedBytes, bytes::Bytes};
use alloy_sol_types::SolCall;
use forker::*;
use rain_interpreter_bindings::DeployerISP::iParserCall;
use rain_interpreter_bindings::IExpressionDeployerV3::deployExpression2Call;
use rain_interpreter_bindings::IParserV1::parseCall;

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
            let block_number = client.get_block_number().await?;

            block_number
        }
    };
    let cache_key = FORKED_EVM_CACHE
        .get_or_create(rpc_url, block_number_val)
        .await?;

    // Call deployer contract: iParserCall
    let calldata = iParserCall {}.abi_encode();
    let response = FORKED_EVM_CACHE
        .call(cache_key.clone(), SENDER_ADDRESS, deployer, &calldata)
        .await??;
    let parser_address = Address::from_word(FixedBytes::from_slice(response.as_ref()));

    // Call parser contract: parseCall
    let calldata = parseCall {
        data: rainlang.as_bytes().to_vec(),
    }
    .abi_encode();
    let expression_config = FORKED_EVM_CACHE
        .call(cache_key.clone(), SENDER_ADDRESS, parser_address, &calldata)
        .await??;

    // Call deployer: deployExpression2Call
    let mut calldata = deployExpression2Call::SELECTOR.to_vec();
    calldata.extend_from_slice(&expression_config);
    FORKED_EVM_CACHE
        .call(cache_key, SENDER_ADDRESS, deployer, &calldata)
        .await??;
    let expression_config_bytes = Bytes::from(expression_config);

    Ok(expression_config_bytes)
}
