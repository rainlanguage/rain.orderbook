use alloy_dyn_abi::JsonAbiExt;
use alloy_json_abi::Error;
use forker::*;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::{collections::HashMap, sync::Mutex};

/// static hashmap of fork evm instances, used for caching instances between runs
pub static FORKS: Lazy<Mutex<HashMap<String, ForkedEvm>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
pub async fn fork_call(
    fork_url: String,
    fork_block_number: u64,
    gas_limit: u64,
    from: Address,
    to: Address,
    calldata: Bytes,
    value: U256,
) -> Result<Result<(), String>, String> {
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
            .call_raw(from, to, calldata, value)
            .map_err(|e| e.to_string())?
    };

    if result.reverted {
        // decode result bytes to error selectors if it was a revert
        Err(decode_error(&result.result).await?)
    } else {
        Ok(Ok(()))
    }
}

/// decodes an error returned from calling a contract by searching its selector in registry
async fn decode_error(error_data: &[u8]) -> Result<String, String> {
    let url = "https://api.openchain.xyz/signature-database/v1/lookup";
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
