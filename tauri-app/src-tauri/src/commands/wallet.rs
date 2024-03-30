use crate::error::CommandResult;
use alloy_ethers_typecast::client::{LedgerClient, HDPath};
use alloy_primitives::Address;

#[tauri::command]
pub async fn get_address_from_ledger(
    chain_id: u64,
    derivation_index: usize,
) -> CommandResult<Address> {
    Ok(
        LedgerClient::get_derivation_address(
            chain_id, HDPath::LedgerLive(derivation_index)
        ).await?
    )
}
