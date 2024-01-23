use forker::ForkedEvm;
use revm::primitives::Bytes;
use revm::primitives::U256;
use revm::primitives::Address;
use std::collections::HashMap;
use foundry_evm::executor::RawCallResult;

#[tauri::command]
pub async fn fork_call(fork_url: String, fork_block_number: Option<u64>, gas_limit: u64, from: Address, to: Address, calldata: Bytes, value: U256) -> Result<RawCallResult, String> {
    let forked_evm = ForkedEvm::new(None, fork_url, fork_block_number, gas_limit);
    forked_evm.call_raw(from, to, calldata, value).map_err(|e| e.to_string());
    result
}