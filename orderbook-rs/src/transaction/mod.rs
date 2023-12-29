use alloy_primitives::{Address, U256};

// use anyhow::anyhow;
use ethers::prelude::SignerMiddleware;
use ethers::types::TransactionReceipt;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Eip1559TransactionRequest, H160, U64},
    utils::parse_units,
};
use ethers_signers::Ledger;
use std::str::FromStr;
use tracing::{info, warn};

use crate::errors::RainOrderbookError;
use crate::gasoracle::gas_price_oracle;

/// Sign and submit transaction on chain via [Ledger] wallet.
///
/// # Arguments
/// * `tx_data` - Abi encoded transaction data, encoded with the function selector.
/// * `tx_to` - [Eip1559TransactionRequest::to]
/// * `tx_value` - [Eip1559TransactionRequest::value]
/// * `rpc_url` - Network RPC
/// * `wallet` - [Ledger] wallet instance
/// * `blocknative_api_key` - Optional Blocknative API Key.
///
pub async fn execute_transaction(
    tx_data: Vec<u8>,
    tx_to: Address,
    tx_value: U256,
    rpc_url: String,
    wallet: Ledger,
    blocknative_api_key: Option<String>,
) -> Result<TransactionReceipt, RainOrderbookError> {
    let provider = Provider::<Http>::try_from(rpc_url.clone())?;

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();
    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;

    let to_address = H160::from_str(&tx_to.to_string()).unwrap();

    let mut tx = Eip1559TransactionRequest::new();

    tx.to = Some(to_address.into());
    tx.value = Some(ethers::types::U256::from_dec_str(tx_value.to_string().as_str()).unwrap());
    tx.data = Some(ethers::types::Bytes::from(tx_data));
    tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap());

    match gas_price_oracle(blocknative_api_key, chain_id).await {
        Ok((max_priority, max_fee)) => {
            let max_priority: ethers::types::U256 =
                parse_units(max_priority.to_string(), 9).unwrap().into();
            let max_fee: ethers::types::U256 = parse_units(max_fee.to_string(), 9).unwrap().into();

            tx.max_priority_fee_per_gas = Some(max_priority);
            tx.max_fee_per_gas = Some(max_fee);
        }
        Err(_) => {
            warn!("BLOCKNATIVE UNSUPPORTED NETWORK");
        }
    };

    let pending_tx = client.send_transaction(tx, None).await?;

    info!("Transaction submitted. Awaiting block confirmations...");
    let tx_confimration = pending_tx.confirmations(1).await?; 
    
    let tx_receipt = match tx_confimration {
        Some(receipt) => receipt,
        None => return Err(RainOrderbookError::TransactionReceiptError),
    };
    info!("Transaction Confirmed!!");
    info!(
        "✅ Hash : 0x{}",
        hex::encode(tx_receipt.transaction_hash.as_bytes())
    );
    Ok(tx_receipt)

    
}
