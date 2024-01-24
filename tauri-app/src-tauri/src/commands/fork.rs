use alloy_dyn_abi::JsonAbiExt;
use alloy_json_abi::Error;
use forker::*;
use once_cell::sync::Lazy;
use reqwest::Client;
use revm::primitives::Bytes;
use std::{collections::HashMap, sync::Mutex};

/// static hashmap of fork evm instances, used for caching instances between runs
pub static FORKS: Lazy<Mutex<HashMap<String, ForkedEvm>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
pub async fn fork_call(
    fork_url: String,
    fork_block_number: u64,
    from_address: &[u8],
    to_address: &[u8],
    calldata: &[u8],
    gas_limit: u64,
) -> Result<Result<Bytes, String>, String> {
    let result = {
        // lock static FORKS
        let mut forks = FORKS.lock().unwrap();

        // build key from fork url and block number
        let key = fork_url.clone() + &fork_block_number.to_string();

        // fork from the provided url, if it is cached, use it, if not create it, and cache it in FORKS
        let forked_evm = if let Some(v) = forks.get_mut(&key) {
            v
        } else {
            let new_forked_evm = ForkedEvm::new(None, fork_url, Some(fork_block_number), gas_limit);
            forks.insert(key.clone(), new_forked_evm);
            forks.get_mut(&key).unwrap()
        };

        // call a contract read-only
        forked_evm
            .call(from_address, to_address, calldata)
            .map_err(|e| e.to_string())?
    };

    println!("{:?}", result);
    if result.reverted {
        // decode result bytes to error selectors if it was a revert
        Err(decode_error(&result.result).await?)
    } else {
        Ok(Ok(result.result))
    }
    // Ok(Ok(()))
}

/// decodes an error returned from calling a contract by searching its selector in registry
async fn decode_error(error_data: &[u8]) -> Result<String, String> {
    let url = "https://api.openchain.xyz/signature-database/v1/lookup";
    println!("{:?}", error_data);
    let (selector_hash_bytes, args_data) = error_data.split_at(4);
    let selector_hash = alloy_primitives::hex::encode_prefixed(selector_hash_bytes);

    let client = Client::builder().build().unwrap();
    let res = client
        .get(url)
        .query(&vec![
            ("function", selector_hash.as_str()),
            ("filter", "true"),
        ])
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(selectors) = res["result"]["function"][selector_hash].as_array() {
        for opt_selector in selectors {
            if let Some(selector) = opt_selector["name"].as_str() {
                if let Ok(error) = selector.parse::<Error>() {
                    if let Ok(result) = error.abi_decode_input(args_data, false) {
                        return Ok(format!("{}: {:?}", error.name, result));
                    }
                }
            }
        }
        Ok("unknown error".to_owned())
    } else {
        Ok("unknown error".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex::decode;
    use alloy_sol_types::SolValue;

    #[tokio::test]
    async fn test_error_decoder() {
        let x = decode_error(&[26, 198, 105, 8]).await;
        assert_eq!(Ok("UnexpectedOperandValue: []".to_owned()), x);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fork_call_parse() {
        // deployer_address 0x5155cE66E704c5Ce79a0c6a1b79113a6033a999b
        // parser_address 0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02
        // some account as caller 0x5855A7b48a1f9811392B89F18A8e27347EF84E42

        let fork_url = "https://rpc.ankr.com/polygon_mumbai".to_owned();
        let fork_block_number = 45122616u64;
        let gas_limit = 200000u64;

        let deployer_address = decode("0x5155cE66E704c5Ce79a0c6a1b79113a6033a999b").unwrap();
        let parser_address = decode("0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02").unwrap();
        let from_address = decode("0x5855A7b48a1f9811392B89F18A8e27347EF84E42").unwrap();

        let rainlang_text = r"_: int-add(1)";
        let mut calldata = decode("0xfab4087a").unwrap(); // parse() selector
        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text

        // this is calling parse() that will not run run integrity checks
        // in order to run integrity checks another call should be done on
        // expressionDeployer2() of deployer contract with same process
        let result = fork_call(
            fork_url.clone(),
            fork_block_number,
            &from_address,
            &parser_address,
            &calldata,
            gas_limit,
        )
        .await;
        let expected = Err("MissingFinalSemi: [Uint(0x000000000000000000000000000000000000000000000000000000000000000d_U256, 256)]".to_owned());
        assert_eq!(result, expected);

        
        // fixed semi error, but still has bad input problem
        // get expressionconfig and call deployer to get integrity checks error
        let rainlang_text = r"_: int-add(1);";
        let mut calldata = decode("0xfab4087a").unwrap(); // parse() selector
        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text
        let result = fork_call(
            fork_url.clone(),
            fork_block_number,
            &from_address,
            &parser_address,
            &calldata,
            gas_limit,
        )
        .await
        .unwrap()
        .unwrap();

        let mut calldata = decode("0xb7f14403").unwrap(); // deployExpression2(bytes,uint256[]) selector
        calldata.extend_from_slice(&result); // extend with result of parse() which is expressionConfig

        // get integrity check results, if not error indicates that text has no error
        // if ends with error, decode with the selectors
        let result = fork_call(
            fork_url,
            fork_block_number,
            &from_address,
            &deployer_address,
            &calldata,
            gas_limit,
        )
        .await;
        let expected = Err("BadOpInputsLength: [Uint(0x0000000000000000000000000000000000000000000000000000000000000001_U256, 256), Uint(0x0000000000000000000000000000000000000000000000000000000000000002_U256, 256), Uint(0x0000000000000000000000000000000000000000000000000000000000000001_U256, 256)]".to_owned());
        assert_eq!(result, expected);
    }
}
