use crate::error::{CommandError, CommandResult};
use alloy::primitives::Address;
use alloy_ethers_typecast::{
    client::{HDPath, LedgerClient},
    ethers_address_to_alloy,
};

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpcs: Vec<String>,
) -> CommandResult<Address> {
    let mut err: Option<CommandError> = None;
    for rpc in rpcs {
        let ledger_client = LedgerClient::new(
            derivation_index.map(HDPath::LedgerLive),
            chain_id,
            rpc,
            None,
        )
        .await;
        match ledger_client {
            Ok(ledger_client) => {
                let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
                return Ok(ledger_address);
            }
            Err(e) => {
                err = Some(CommandError::LedgerClientError(e));
            }
        }
    }
    // If we get here, all rpcs failed
    Err(err.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_address_from_ledger_err() {
        // NOTE: the error is different depending on whether a ledger is connected or not
        let _ = get_address_from_ledger(None, 1, vec!["this is a bad a url".to_string()])
            .await
            .unwrap_err();
    }

    // NOTE: we can't mock a ledger connection, so we can't test the ok case
}
