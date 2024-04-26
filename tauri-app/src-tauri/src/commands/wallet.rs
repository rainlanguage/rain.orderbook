use crate::error::{CommandResult};
use alloy_ethers_typecast::{client::LedgerClient, ethers_address_to_alloy};
use alloy_primitives::{Address};
use ethers::middleware::Middleware;
use ethers::providers::{Provider, Http};
use ethers::utils::format_units;

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpc_url: String,
) -> CommandResult<Address> {
    let ledger_client =
        LedgerClient::new(derivation_index, chain_id, rpc_url.clone(), None).await?;
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());

    Ok(ledger_address)
}

#[tauri::command]
pub async fn get_balance_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpc_url: String,
) -> CommandResult<String> {
    let ledger_client =
        LedgerClient::new(derivation_index, chain_id, rpc_url.clone(), None).await?;
    let address = ledger_client.client.address();
    let provider = Provider::<Http>::try_from(rpc_url).expect("Invalid RPC URL");

    let balance = provider.get_balance(address, None).await?;

    let balance_display = format_units(balance, 18).unwrap_or_else(|_| "Invalid balance".to_string());

    if cfg!(debug_assertions) {
        println!("Ledger balance_display: {}", balance_display);
    }

    Ok(balance_display)
}

#[tauri::command]
pub async fn get_balance_from_wallet(
    address: String,
    rpc_url: String,
) -> CommandResult<String> {
    let provider = Provider::<Http>::try_from(rpc_url).expect("Invalid RPC URL");
    let address = address.parse::<ethers::types::Address>().expect("Invalid address");
    let balance = provider.get_balance(address, None).await?;

    let balance_display = format_units(balance, 18).unwrap_or_else(|_| "Invalid balance".to_string());

    if cfg!(debug_assertions) {
        println!("Wallet balance_display: {}", balance_display);
    }

    Ok(balance_display)
}

