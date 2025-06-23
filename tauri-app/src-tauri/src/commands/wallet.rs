use crate::error::CommandResult;
use alloy::primitives::Address;
use alloy::signers::ledger::{HDPath, LedgerSigner};

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
) -> CommandResult<Address> {
    let signer = if let Some(derivation_index) = derivation_index {
        LedgerSigner::new(HDPath::LedgerLive(derivation_index), Some(chain_id)).await?
    } else {
        LedgerSigner::new(HDPath::LedgerLive(0), Some(chain_id)).await?
    };
    let address = signer.get_address().await?;
    Ok(address)
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
