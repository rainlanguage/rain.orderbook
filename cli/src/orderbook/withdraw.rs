use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units, prelude::SignerMiddleware} ; 
use ethers_signers::Ledger;
use std::{convert::TryFrom, sync::Arc};
use spinners::{Spinner, Spinners};
use std::str::FromStr;
use anyhow::anyhow;

use crate::{cli::registry::{IOrderBookV2, WithdrawConfig}, gasoracle::{is_block_native_supported, gas_price_oracle}}; 

pub async fn withdraw_tokens(
    withdraw_token_address : H160 ,
    withdraw_token_amount : U256 ,
    wihtdraw_vault_id : U256,
    orderbook_address : H160,
    rpc_url : String,
    wallet: Ledger,
    blocknative_api_key : Option<String>
) -> anyhow::Result<()>{ 

    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 

    let signer_address =  wallet.get_address().await.unwrap()  ;
    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();

    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(provider.clone()));  

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

    let withdraw_tx = orderbook.withdraw(withdraw_config) ; 
    let withdraw_data: Bytes = withdraw_tx.calldata().unwrap() ;

    let mut withdraw_tx = Eip1559TransactionRequest::new();
    withdraw_tx.to = Some(orderbook_address.into());
    withdraw_tx.value = Some(U256::zero());
    withdraw_tx.data = Some(withdraw_data);
    withdraw_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 

    if is_block_native_supported(chain_id) {
        let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

        withdraw_tx.max_priority_fee_per_gas = Some(max_priority);
        withdraw_tx.max_fee_per_gas = Some(max_fee);
    }
    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;   

    println!("\n-----------------------------------\nWithdrawing tokens from vault.\n");
    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Awaiting confirmation from wallet...".into(),
    );  
    let withdraw_tx = client.send_transaction(withdraw_tx, None).await;   
    sp.stop();

    match withdraw_tx {
        Ok(withdraw_tx) => {
            let mut sp = Spinner::new(
                Spinners::from_str("Dots9").unwrap(),
                "Transaction submitted. Awaiting block confirmations...".into(),
            );
            let withdraw_receipt = withdraw_tx.confirmations(1).await?.unwrap();  
            let withdraw_msg = format!(
                "{}{}{}" ,
                String::from("\nTokens withdrawn !!\n#################################\n✅ Hash : "),
                format!("0x{}",hex::encode(withdraw_receipt.transaction_hash.as_bytes().to_vec())), 
                String::from("\n-----------------------------------\n")
            ) ; 
            sp.stop_with_message(withdraw_msg.into()); 
        }
        Err(_) => {
            println!("\n❌ Transaction Rejected.");
        }
    } 
    Ok(())
}