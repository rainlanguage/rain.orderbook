use alloy_ethers_typecast::client::LedgerClient;
use alloy_ethers_typecast::ethers_address_to_alloy;
use alloy_primitives::Address;
use anyhow::Result;

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpc_url: String,
) -> Result<Address> {
    let ledger_client = LedgerClient::new(derivation_index, chain_id, rpc_url.clone())
        .await?;
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());

    Ok(ledger_address)
}
