use alloy_ethers_typecast::client::LedgerClient;
use alloy_ethers_typecast::ethers_address_to_alloy;
use alloy_primitives::Address;

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpc_url: String,
) -> Result<Address, String> {
    let ledger_client = LedgerClient::new(derivation_index, chain_id, rpc_url.clone())
        .await
        .map_err(|e| format!("error is {}", e.to_string()))?;
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());

    Ok(ledger_address)
}
