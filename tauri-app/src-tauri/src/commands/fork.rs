use forker::*;

#[tauri::command]
pub async fn fork_call(fork_url: String, fork_block_number: Option<u64>, gas_limit: u64, from: Address, to: Address, calldata: Bytes, value: U256) -> Result<Bytes, String> {

    // fork from the provided url
    let mut forked_evm = ForkedEvm::new(None, fork_url, fork_block_number, gas_limit);

    // call a contract read-only
    let result = forked_evm.call_raw(from, to, calldata, value).map_err(|e| e.to_string())?;

    if result.reverted {
        // decode result bytes to error selectors if it was a revert
        Err(String::new())
    } else {
        Ok(result.result)
    }
}