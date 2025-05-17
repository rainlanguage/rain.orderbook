use crate::error::CommandResult;
use alloy::primitives::Address;
use alloy_ethers_typecast::{client::HDPath, client::LedgerClient, ethers_address_to_alloy};

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpc_url: String,
) -> CommandResult<Address> {
    let ledger_client = LedgerClient::new(
        derivation_index.map(HDPath::LedgerLive),
        chain_id,
        rpc_url.clone(),
        None,
    )
    .await?;
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());

    Ok(ledger_address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_address_from_ledger_err() {
        // NOTE: the error is different depending on whether a ledger is connected or not
        let _ = get_address_from_ledger(None, 1, "this is a bad a url".to_string())
            .await
            .unwrap_err();
    }

    // NOTE: we can't mock a ledger connection, so we can't test the ok case
}
