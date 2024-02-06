use super::error::ForkCallError;
use super::error::{abi_decode_error, AbiDecodedErrorType};
use crate::add_order::AddOrderArgs;
use crate::error::ForkParseError;
use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use forker::*;
use once_cell::sync::Lazy;
use rain_interpreter_bindings::DeployerISP::iParserCall;
use rain_interpreter_bindings::IExpressionDeployerV3::deployExpression2Call;
use rain_interpreter_bindings::IParserV1::parseCall;
use revm::primitives::Bytes;
use std::{collections::HashMap, sync::Mutex};

const FROM_ADDRESS: Address = Address::repeat_byte(0x1);

/// Cache of evm fork instances, keyed by rpc url + block number
pub static FORKS: Lazy<Mutex<HashMap<String, ForkedEvm>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn fork_call(
    rpc_url: &str,
    block_number: u64,
    sender: Address,
    address: Address,
    calldata: &[u8],
) -> Result<Result<Bytes, AbiDecodedErrorType>, ForkCallError> {
    // build cache key from fork url and block number
    let key = rpc_url.to_owned() + &block_number.to_string();

    let is_cached = FORKS.lock()?.contains_key(&key);
    let result = if is_cached {
        // load evm fork from cache
        let mut forks = FORKS.lock()?;
        let forked_evm = forks
            .get_mut(&key)
            .ok_or(ForkCallError::ForkCacheKeyMissing(key))?;

        // call contract on evm fork
        forked_evm
            .call(sender.as_slice(), address.as_slice(), calldata)
            .map_err(|e| ForkCallError::EVMError(e.to_string()))?
    } else {
        // Generate new evm fork
        let mut forked_evm =
            ForkedEvm::new(rpc_url, Some(block_number), Some(200000u64), None).await;

        // call contract on evm fork
        let result = forked_evm
            .call(sender.as_slice(), address.as_slice(), calldata)
            .map_err(|e| ForkCallError::EVMError(e.to_string()))?;

        // add new fork to cache
        let mut forks = FORKS.lock()?;
        forks.insert(key, forked_evm);

        result
    };

    if result.reverted {
        // decode result bytes to error selectors if it was a revert
        Ok(Err(abi_decode_error(&result.result).await?))
    } else {
        Ok(Ok(result.result))
    }
}

/// checks the front matter validity and parses the given rainlang string
/// with the deployer parsed from the front matter
/// returns abi encoded expression config on Ok variant
pub async fn fork_parse_rainlang(
    frontmatter: &str,
    rainlang: &str,
    rpc_url: &str,
    block_number: u64,
) -> Result<Bytes, ForkParseError> {
    let deployer = AddOrderArgs::try_parse_frontmatter(frontmatter)?.0;

    let calldata = iParserCall {}.abi_encode();
    let parser = fork_call(rpc_url, block_number, FROM_ADDRESS, deployer, &calldata).await??;
    let parser_bytes =
        slice_as_array!(parser.as_ref(), [u8; 20]).ok_or(ForkParseError::ParserAddressInvalid)?;
    let parser_address = Address::from(parser_bytes);

    let calldata = parseCall {
        data: rainlang.as_bytes().to_vec(),
    }
    .abi_encode();
    let expression_config = fork_call(
        rpc_url,
        block_number,
        FROM_ADDRESS,
        parser_address,
        &calldata,
    )
    .await??;

    let mut calldata = deployExpression2Call::SELECTOR.to_vec();
    calldata.extend_from_slice(&expression_config);
    fork_call(rpc_url, block_number, FROM_ADDRESS, deployer, &calldata).await??;

    Ok(expression_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex::decode;
    use alloy_sol_types::SolValue;

    const FORK_URL: &str = "https://rpc.ankr.com/polygon_mumbai";
    const FORK_BLOCK_NUMBER: u64 = 45122616;
    const FROM_ADDRESS: Address = Address::repeat_byte(0x1);
    const DEPLOY_EXPRESSION_2_SELECTOR: &str = "0xb7f14403"; // deployExpression2(bytes,uint256[])
    const PARSE_SELECTOR: &str = "0xfab4087a"; // parse()

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fork_call_parse_fail_parse() {
        // has no semi at the end
        let rainlang_text = r"_: int-add(1)";
        let mut calldata = decode(PARSE_SELECTOR).unwrap();
        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text to build calldata
        let parser_address: Address = "0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02"
            .parse::<Address>()
            .unwrap();

        // this is calling parse() that will not run integrity checks
        // in order to run integrity checks another call should be done on
        // expressionDeployer2() of deployer contract with same process
        let result = fork_call(
            FORK_URL,
            FORK_BLOCK_NUMBER,
            FROM_ADDRESS,
            parser_address,
            &calldata,
        )
        .await
        .expect("test failed!");
        let expected = Err(AbiDecodedErrorType::Known {
            name: "MissingFinalSemi".to_owned(),
            args: vec!["Uint(0x000000000000000000000000000000000000000000000000000000000000000d_U256, 256)".to_owned()],
            sig: "MissingFinalSemi(uint256)".to_owned(),
        });
        assert_eq!(result, expected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fork_call_parse_fail_integrity() {
        // fixed semi error, but still has bad input problem
        // get expressionconfig and call deployer to get integrity checks error
        let rainlang_text = r"_: int-add(1);";
        let mut calldata = decode(PARSE_SELECTOR).unwrap();
        let parser_address: Address = "0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02"
            .parse::<Address>()
            .unwrap();

        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text
        let expression_config = fork_call(
            FORK_URL,
            FORK_BLOCK_NUMBER,
            FROM_ADDRESS,
            parser_address,
            &calldata,
        )
        .await
        .unwrap()
        .unwrap();

        let mut calldata = decode(DEPLOY_EXPRESSION_2_SELECTOR).unwrap();
        calldata.extend_from_slice(&expression_config); // extend with result of parse() which is expressionConfig
        let deployer_address: Address = "0x5155cE66E704c5Ce79a0c6a1b79113a6033a999b"
            .parse::<Address>()
            .unwrap();

        // get integrity check results, if ends with error, decode with the selectors
        let result = fork_call(
            FORK_URL,
            FORK_BLOCK_NUMBER,
            FROM_ADDRESS,
            deployer_address,
            &calldata,
        )
        .await
        .expect("test failed!");
        let expected = Err(AbiDecodedErrorType::Known {
            name: "BadOpInputsLength".to_owned(),
            args: vec![
                "Uint(0x0000000000000000000000000000000000000000000000000000000000000001_U256, 256)".to_owned(), 
                "Uint(0x0000000000000000000000000000000000000000000000000000000000000002_U256, 256)".to_owned(), 
                "Uint(0x0000000000000000000000000000000000000000000000000000000000000001_U256, 256)".to_owned()
            ],
            sig: "BadOpInputsLength(uint256,uint256,uint256)".to_owned(),
        });
        assert_eq!(result, expected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fork_call_parse_success() {
        // get expressionconfig and call deployer to get integrity checks error
        let rainlang_text = r"_: int-add(1 2);";
        let mut calldata = decode(PARSE_SELECTOR).unwrap();
        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text
        let parser_address: Address = "0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02"
            .parse::<Address>()
            .unwrap();

        let expression_config = fork_call(
            FORK_URL,
            FORK_BLOCK_NUMBER,
            FROM_ADDRESS,
            parser_address,
            &calldata,
        )
        .await
        .unwrap()
        .unwrap();

        let mut calldata = decode(DEPLOY_EXPRESSION_2_SELECTOR).unwrap();
        calldata.extend_from_slice(&expression_config); // extend with result of parse() which is expressionConfig
        let deployer_address: Address = "0x5155cE66E704c5Ce79a0c6a1b79113a6033a999b"
            .parse::<Address>()
            .unwrap();

        // expression deploys ok so the expressionConfig in previous step can be used to deploy onchain
        let result = fork_call(
            FORK_URL,
            FORK_BLOCK_NUMBER,
            FROM_ADDRESS,
            deployer_address,
            &calldata,
        )
        .await
        .expect("test failed");
        let expected = Ok(
            Bytes::from(
                alloy_primitives::hex::decode(
                    "0x000000000000000000000000f22cda7695125487993110d706f3b001c8d106400000000000000000000000008a326d777bc34ea563bd21854b436d458112185b00000000000000000000000064c9b10e815a089698521b10be95c8c9c2ed0b3c000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000020001000000000000000000000000000000000000000000000000000000000000").unwrap()
                )
            );
        assert_eq!(result, expected);
    }
}
