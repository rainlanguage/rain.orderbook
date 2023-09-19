use anyhow::anyhow;
use ethers::prelude::SignerMiddleware;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::Eip1559TransactionRequest,
};
use ethers_signers::Ledger;
use tracing::{error, info};

/// Submits [Eip1559TransactionRequest] to the blockchain via the signer.
///
/// # Arguments
/// * `rpc_url` - Provider RPC URL.
/// * `wallet` - Ledger wallet instance.
/// * `transaction` - [Eip1559TransactionRequest] instance to be submitted to the blockchain.
pub async fn execute_transaction(
    rpc_url: String,
    wallet: Ledger,
    transaction: Eip1559TransactionRequest,
) -> anyhow::Result<()> {
    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            error!("INVALID RPC URL: {}", err);
            return Err(anyhow!(err));
        }
    };

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;

    info!("Awaiting confirmation from wallet...");
    let approve_tx = client.send_transaction(transaction, None).await;

    match approve_tx {
        Ok(approve_tx) => {
            info!("Transaction submitted. Awaiting block confirmations...");
            let approve_receipt = match approve_tx.confirmations(1).await {
                Ok(receipt) => match receipt {
                    Some(receipt) => receipt,
                    None => {
                        error!("FAILED TO FETCH RECEIPT");
                        return Err(anyhow!("Failed to fetch receipt."));
                    }
                },
                Err(err) => {
                    error!("FAILED TO CONFIRM TRANSACTION : {}", err);
                    return Err(anyhow!(err));
                }
            };
            info!("Transaction Confirmed!!");
            info!(
                "✅ Hash : 0x{}",
                hex::encode(approve_receipt.transaction_hash.as_bytes().to_vec())
            );
        }
        Err(err) => {
            error!("TRANSACTION REJECTED : {}", err);
        }
    }

    Ok(())
}
