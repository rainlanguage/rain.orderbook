use ethers::{types::{H160, U256}, prelude::SignerMiddleware};
use ethers_signers::Ledger;
use ethers::providers::{Provider,Http} ;
use std::{convert::TryFrom, sync::Arc};
use spinners::{Spinner, Spinners};
use std::str::FromStr;
use anyhow::anyhow;

use crate::cli::registry::{IOrderBookV2, WithdrawConfig}; 

pub async fn withdraw_tokens(
    withdraw_token_address : H160 ,
    withdraw_token_amount : U256 ,
    wihtdraw_vault_id : U256,
    orderbook_address : H160,
    rpc_url : String,
    wallet: Ledger

) -> anyhow::Result<()>{ 

    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?; 
    let signer_address =  client.address() ;

    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(client));  

    let vault_balance: U256 = orderbook.vault_balance(signer_address, withdraw_token_address, wihtdraw_vault_id).call().await.unwrap() ; 

    if withdraw_token_amount.gt(&vault_balance) { 

        let err_msg = format!(
            "{}{}" ,
            String::from("\n#################################\nInsufficient vault balance for withdrawal.\nCurrent vault balance :"),
            vault_balance.to_string()
        ) ;  
        println!("{}",err_msg) ;
        return Err(anyhow!(err_msg)); 
    }

    let withdraw_config = WithdrawConfig{
        token : withdraw_token_address ,
        vault_id : wihtdraw_vault_id ,
        amount : withdraw_token_amount
    } ; 

    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Withdrawing tokens from the vault...".into(),
    ); 

    let withdraw_tx = orderbook.withdraw(withdraw_config) ;
    let withdraw_pending_tx = withdraw_tx.send().await?;
    let withdraw_receipt = withdraw_pending_tx.confirmations(3).await?.unwrap(); 

    let withdraw_msg = format!(
        "{}{}{}" ,
        String::from("\nTokens withdrawn !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(withdraw_receipt.transaction_hash.as_bytes().to_vec())), 
        String::from("\n-----------------------------------\n")
    ) ; 
    sp.stop_with_message(withdraw_msg.into()); 

    Ok(())
}