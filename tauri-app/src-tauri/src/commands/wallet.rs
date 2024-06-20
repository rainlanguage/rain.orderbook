use crate::error::CommandResult;
use alloy_ethers_typecast::{client::{LedgerClient, HDPath}, ethers_address_to_alloy};
use alloy_primitives::Address;

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpc_url: String,
) -> CommandResult<Address> {
    let ledger_client =
        LedgerClient::new(derivation_index.map(HDPath::LedgerLive), chain_id, rpc_url.clone(), None).await?;
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());

    Ok(ledger_address)
}
