use crate::error::{CommandError, CommandResult};
use alloy::primitives::Address;
use alloy::signers::ledger::{HDPath, LedgerSigner};

#[tauri::command]
pub async fn get_address_from_ledger(
    derivation_index: Option<usize>,
    chain_id: u64,
    rpcs: Vec<String>,
) -> CommandResult<Address> {
    let mut err: Option<CommandError> = None;
    if rpcs.is_empty() {
        return Err(CommandError::MissingRpcs);
    }

    let derivation_index = derivation_index.unwrap_or(0);

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
                let signer =
                    LedgerSigner::new(HDPath::LedgerLive(derivation_index), Some(chain_id)).await?;
                let address = signer.get_address().await?;
                return Ok(address);
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
