use alloy_primitives::{Address, U256};

use anyhow::anyhow;
use ethers::prelude::SignerMiddleware;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{ Eip1559TransactionRequest, H160, U64},
    utils::parse_units,
};
use ethers_signers::Ledger;
use std::str::FromStr;
use tracing::{error, info};

use crate::gasoracle::{is_block_native_supported, gas_price_oracle};

pub async fn execute_transaction(
    tx_data: Vec<u8>,
    tx_to: Address,
    tx_value: U256,
    rpc_url: String,
    wallet: Ledger,
    blocknative_api_key: Option<String>
) -> anyhow::Result<()> {

    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            error!("INVALID RPC URL: {}", err);
            return Err(anyhow!(err));
        }
    };

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64(); 
    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;

    let to_address = H160::from_str(&tx_to.to_string()).unwrap();

    let mut tx = Eip1559TransactionRequest::new(); 

    tx.to = Some(to_address.into());
    tx.value = Some(ethers::types::U256::from_dec_str(tx_value.to_string().as_str()).unwrap());
    tx.data = Some(ethers::types::Bytes::from(tx_data));
    tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap());

    if is_block_native_supported(chain_id) {
        let (max_priority, max_fee) = gas_price_oracle(blocknative_api_key, chain_id)
            .await
            .unwrap();
        let max_priority: ethers::types::U256 = parse_units(max_priority.to_string(), 9).unwrap().into();
        let max_fee: ethers::types::U256 = parse_units(max_fee.to_string(), 9).unwrap().into();

        tx.max_priority_fee_per_gas = Some(max_priority);
        tx.max_fee_per_gas = Some(max_fee);
    }

    let tx_result = client.send_transaction(tx, None).await;

    match tx_result {
        Ok(tx_result) => {
            info!("Transaction submitted. Awaiting block confirmations...");
            let approve_receipt = match tx_result.confirmations(1).await {
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