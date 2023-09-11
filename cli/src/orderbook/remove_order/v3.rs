use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units, prelude::SignerMiddleware} ; 
use std::{convert::TryFrom, sync::Arc};
use ethers_signers::Ledger;
use spinners::{Spinner, Spinners};
use std::str::FromStr;

use crate::{cli::registry::{Order, IOrderBookV3}, gasoracle::{is_block_native_supported, gas_price_oracle}}; 


pub async fn remove_order(
    order_to_remove : Order ,  
    orderbook_address : H160 ,
    rpc_url : String ,
    wallet : Ledger ,
    blocknative_api_key : Option<String>
) -> anyhow::Result<()> {

    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 
    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();

    let orderbook = IOrderBookV3::new(orderbook_address.clone(), Arc::new(provider.clone())); 

    let remove_order_tx = orderbook.remove_order(order_to_remove) ; 

    let remove_order_data: Bytes = remove_order_tx.calldata().unwrap() ;

    let mut remove_order_tx = Eip1559TransactionRequest::new();
    remove_order_tx.to = Some(orderbook_address.into());
    remove_order_tx.value = Some(U256::zero());
    remove_order_tx.data = Some(remove_order_data);
    remove_order_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 

    if is_block_native_supported(chain_id) {
        let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

        remove_order_tx.max_priority_fee_per_gas = Some(max_priority);
        remove_order_tx.max_fee_per_gas = Some(max_fee);
    }
    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;

    println!("\n-----------------------------------\nRemoving Orders\n");
    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Awaiting confirmation from wallet...".into(),
    );  
    let remove_order_tx = client.send_transaction(remove_order_tx, None).await;   
    sp.stop() ;

    match remove_order_tx {
        Ok(remove_order_tx) => {
            let mut sp = Spinner::new(
                Spinners::from_str("Dots9").unwrap(),
                "Transaction submitted. Awaiting block confirmations...".into(),
            );
            let remove_order_receipt = remove_order_tx.confirmations(1).await?.unwrap();  
            let order_msg = format!(
                "{}{}{}" ,
                String::from("\nOrder Removed !!\n#################################\n✅ Hash : "),
                format!("0x{}",hex::encode(remove_order_receipt.transaction_hash.as_bytes().to_vec())), 
                String::from("\n-----------------------------------\n")
            ) ; 
            sp.stop_with_message(order_msg.into());   
        }
        Err(_) => {
            println!("\n❌ Transaction Rejected.");
        }
    }  
    Ok(())

}