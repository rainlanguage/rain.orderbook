use crate::error::CommandResult;
use alloy::primitives::Address;
use alloy::signers::ledger::{HDPath, LedgerSigner};

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
) -> CommandResult<Address> {
    let derivation_index = derivation_index.unwrap_or(0);

    let signer = LedgerSigner::new(HDPath::LedgerLive(derivation_index), Some(chain_id)).await?;

    let address = signer.get_address().await?;
    Ok(address)
}

// NOTE: we can't mock a ledger connection, so we can't test add test coverage
