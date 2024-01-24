use forker::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// static hashmap of fork evm instances, used for caching instances between runs
pub static mut FORKS: Lazy<HashMap<String, ForkedEvm>> = Lazy::new(HashMap::new);

#[tauri::command]
pub async fn fork_call(
    fork_url: String,
    fork_block_number: u64,
    gas_limit: u64,
    from: Address,
    to: Address,
    calldata: Bytes,
    value: U256,
) -> Result<Bytes, String> {
    // build key from fork url and block number
    let key = fork_url.clone() + &fork_block_number.to_string();

    // fork from the provided url, if it is cached, use it, if not create it, and cache it in FORKS
    let forked_evm: &mut ForkedEvm;
    unsafe {
        if let Some(v) = FORKS.get_mut(&key) {
            forked_evm = v;
        } else {
            let new_forked_evm = ForkedEvm::new(None, fork_url, Some(fork_block_number), gas_limit);
            FORKS.insert(key.clone(), new_forked_evm);
            forked_evm = FORKS.get_mut(&key).unwrap();
        }
    }

    // call a contract read-only
    let result = forked_evm
        .call_raw(from, to, calldata, value)
        .map_err(|e| e.to_string())?;

    if result.reverted {
        // decode result bytes to error selectors if it was a revert
        Err(String::new())
    } else {
        Ok(result.result)
    }
}
