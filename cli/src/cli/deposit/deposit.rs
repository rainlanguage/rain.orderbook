use ethers::{providers::{Provider, Http}, types::{H160,U256}, prelude::SignerMiddleware} ; 
use ethers_signers::Ledger;
use std::{convert::TryFrom, sync::Arc};
use spinners::{Spinner, Spinners};  
use std::str::FromStr;

use crate::cli::registry::{IOrderBookV2, DepositConfig};

pub async fn deposit_token( 
    deposit_token_address : H160 ,
    deposit_token_amount : U256 ,
    deposit_vault_id : U256,
    orderbook_address : H160 ,
    rpc_url : String ,
    wallet : Ledger
) -> anyhow::Result<()> { 
    
    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");  

    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(SignerMiddleware::new(provider, wallet))); 

    let deposit_config = DepositConfig{
        token : deposit_token_address ,
        vault_id : deposit_vault_id ,
        amount : deposit_token_amount
    } ; 

    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Depositing token in vault...".into(),
    ); 

    let deposit_tx = orderbook.deposit(deposit_config) ;
    let deposit_pending_tx = deposit_tx.send().await?;
    let depsoit_receipt = deposit_pending_tx.confirmations(3).await?.unwrap(); 

    let deposit_msg = format!(
        "{}{}{}{}{}" ,
        String::from("\nTokens deposited in vault !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(depsoit_receipt.transaction_hash.as_bytes().to_vec())), 
        String::from("\nVault Id : "),
        deposit_vault_id ,
        String::from("\n-----------------------------------\n")
    ) ; 
    sp.stop_with_message(deposit_msg.into());  

    Ok(())
}