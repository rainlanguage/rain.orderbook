use std::{convert::TryFrom, sync::Arc};
use ethers::{providers::{Provider, Http}, types::H160} ; 
use ethers::prelude::SignerMiddleware;
use ethers_signers::Ledger;
use spinners::{Spinner, Spinners};
use std::str::FromStr;

use crate::cli::registry::{Order, IOrderBookV2}; 


pub async fn remove_order(
    order_to_remove : Order ,  
    orderbook_address : H160 ,
    rpc_url : String ,
    wallet : Ledger
) -> anyhow::Result<()> {

    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?; 

    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(client)); 

    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Removing Order...".into(),
    ); 

    let remove_order_tx = orderbook.remove_order(order_to_remove) ; 

    let remove_order_pending_tx = remove_order_tx.send().await?;

    let remove_order_receipt = remove_order_pending_tx.confirmations(3).await?.unwrap(); 

    let order_msg = format!(
        "{}{}{}" ,
        String::from("\nOrder Removed !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(remove_order_receipt.transaction_hash.as_bytes().to_vec())), 
        String::from("\n-----------------------------------\n")
    ) ; 
    sp.stop_with_message(order_msg.into()); 

    Ok(())

}