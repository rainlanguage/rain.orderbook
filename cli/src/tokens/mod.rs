
use anyhow::anyhow;
use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units, prelude::SignerMiddleware} ; 
use std::{convert::TryFrom, sync::Arc};
use spinners::{Spinner, Spinners};  
use std::str::FromStr;
use ethers_signers::Ledger; 
use crate::{cli::registry::IERC20, gasoracle::{is_block_native_supported, gas_price_oracle}};  



pub async fn approve_tokens(
    token_address : H160 ,
    token_amount : U256 , 
    approver_address : H160 , 
    rpc_url : String , 
    wallet : Ledger,
    blocknative_api_key : Option<String>
) -> anyhow::Result<()> {  

    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");

    let signer_address =  wallet.get_address().await.unwrap() ; 
    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();  

    let token_contract = IERC20::new(token_address.clone(),Arc::new(provider.clone())) ; 
    let token_balance: U256 = token_contract.balance_of(signer_address).call().await.unwrap() ;  

    if token_balance.gt(&token_amount.clone()) { 
        let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;  

        let approve_tx = token_contract.approve(approver_address.clone(), token_amount.clone()) ; 
        let data: Bytes = approve_tx.calldata().unwrap() ;

        let mut approve_tx = Eip1559TransactionRequest::new();
        approve_tx.to = Some(token_address.into());
        approve_tx.value = Some(U256::zero());
        approve_tx.data = Some(data);
        approve_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 

        if is_block_native_supported(chain_id) {
            let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
            let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
            let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

            approve_tx.max_priority_fee_per_gas = Some(max_priority);
            approve_tx.max_fee_per_gas = Some(max_fee);
        }

        println!("\n-----------------------------------\nApproving token for deposit\n");
        let mut sp = Spinner::new(
            Spinners::from_str("Dots9").unwrap(),
            "Awaiting confirmation from wallet...".into(),
        );  
        let approve_tx = client.send_transaction(approve_tx, None).await;   
        sp.stop() ;

        match approve_tx {
            Ok(approve_tx) => {
                let mut sp = Spinner::new(
                    Spinners::from_str("Dots9").unwrap(),
                    "Transaction submitted. Awaiting block confirmations...".into(),
                );
                let approve_receipt = approve_tx.confirmations(1).await?.unwrap();  
                let end_msg = format!(
                    "{}{}{}" ,
                    String::from("\nTokens Approved for deposit !!\n#################################\n✅ Hash : "),
                    format!("0x{}",hex::encode(approve_receipt.transaction_hash.as_bytes().to_vec())), 
                    String::from("\n-----------------------------------\n")
                ) ; 
                sp.stop_with_message(end_msg.into()); 
            }
            Err(_) => {
                println!("\n❌ Transaction Rejected.");
            }
        }
        
    }else{
        return Err(anyhow!("\n ❌Insufficent balance for deposit.\nCurrent Balance : {}.",token_balance)) ;
    } 

    Ok(())
}