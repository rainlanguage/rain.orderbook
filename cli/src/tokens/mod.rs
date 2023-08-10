
use anyhow::anyhow;
use ethers::{providers::{Provider, Http}, types::{H160,U256},  prelude::SignerMiddleware} ; 
use std::{convert::TryFrom, sync::Arc};
use spinners::{Spinner, Spinners};  
use std::str::FromStr;
use ethers_signers::Ledger;

use crate::cli::registry::IERC20; 

pub async fn approve_tokens(
    token_address : H160 ,
    token_amount : U256 , 
    approver_address : H160 , 
    rpc_url : String , 
    wallet : Ledger
) -> anyhow::Result<()> {  

    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");

    let signer_address =  wallet.get_address().await.unwrap() ;

    let token_contract = IERC20::new(token_address,Arc::new(SignerMiddleware::new(provider, wallet))) ; 
    let token_balance: U256 = token_contract.balance_of(signer_address).call().await.unwrap() ;  

    if token_balance.gt(&token_amount.clone()) {
        let approve_tx = token_contract.approve(approver_address.clone(), token_amount.clone()) ; 
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

    Ok(())
}