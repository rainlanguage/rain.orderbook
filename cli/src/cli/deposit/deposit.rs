use ethers::{providers::{Provider, Http}, types::{H160,U256}, prelude::SignerMiddleware} ; 
use ethers_signers::Ledger;
use std::{convert::TryFrom, sync::Arc};
use spinners::{Spinner, Spinners};  
use std::str::FromStr;
use anyhow::anyhow;

use crate::cli::registry::{IERC20, IOrderBookV2, DepositConfig};

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

    let signer_address =  wallet.get_address().await.unwrap() ;

    let token_contract = IERC20::new(deposit_token_address,Arc::new(provider.clone())) ; 
    let token_balance: U256 = token_contract.balance_of(signer_address).call().await.unwrap() ;  

    if token_balance.gt(&deposit_token_amount.clone()) {
        let approve_tx = token_contract.approve(orderbook_address.clone(), deposit_token_amount.clone()) ; 
        let mut sp = Spinner::new(
            Spinners::from_str("Dots9").unwrap(),
            "Approving tokens for deposit...".into(),
        );  
        let approve_pending_tx = approve_tx.send().await? ;
        let approve_receipt = approve_pending_tx.confirmations(4).await?.unwrap();  

        let end_msg = format!(
            "{}{}{}" ,
            String::from("\nTokens Approved for deposit !!\n#################################\n✅ Hash : "),
            format!("0x{}",hex::encode(approve_receipt.transaction_hash.as_bytes().to_vec())), 
            String::from("\n-----------------------------------\n")
        ) ; 
        sp.stop_with_message(end_msg.into()); 


    }else{
        return Err(anyhow!("\n ❌Insufficent balance for deposit.\nCurrent Balance : {}.",token_balance)) ;
    } 

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
        "{}{}{}" ,
        String::from("\nTokens deposited in vault !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(depsoit_receipt.transaction_hash.as_bytes().to_vec())), 
        String::from("\n-----------------------------------\n")
    ) ; 
    sp.stop_with_message(deposit_msg.into());  

    Ok(())
}