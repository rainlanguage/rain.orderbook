use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units, prelude::SignerMiddleware} ; 
use ethers_signers::Ledger;
use std::{convert::TryFrom, sync::Arc};
use spinners::{Spinner, Spinners};  
use std::str::FromStr;

use crate::{cli::registry::{IOrderBookV2, DepositConfig}, gasoracle::{is_block_native_supported, gas_price_oracle}};

pub async fn deposit_token( 
    deposit_token_address : H160 ,
    deposit_token_amount : U256 ,
    deposit_vault_id : U256,
    orderbook_address : H160 ,
    rpc_url : String ,
    wallet : Ledger ,
    blocknative_api_key : Option<String>
) -> anyhow::Result<()> { 
    
    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");   

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64(); 

    let orderbook = IOrderBookV2::new(orderbook_address.clone(), Arc::new(provider.clone())); 

    let deposit_config = DepositConfig{
        token : deposit_token_address ,
        vault_id : deposit_vault_id ,
        amount : deposit_token_amount
    } ; 

    let deposit_tx = orderbook.deposit(deposit_config) ; 
    let deposit_data: Bytes = deposit_tx.calldata().unwrap() ;

    let mut deposit_tx = Eip1559TransactionRequest::new();
    deposit_tx.to = Some(orderbook_address.into());
    deposit_tx.value = Some(U256::zero());
    deposit_tx.data = Some(deposit_data);
    deposit_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 

    if is_block_native_supported(chain_id) {
        let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

        deposit_tx.max_priority_fee_per_gas = Some(max_priority);
        deposit_tx.max_fee_per_gas = Some(max_fee);
    }
    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;   

    println!("\n-----------------------------------\nDepositing Tokens Into Vaults\n");
    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Awaiting confirmation from wallet...".into(),
    );  
    let deposit_tx = client.send_transaction(deposit_tx, None).await;   
    sp.stop() ;

    match deposit_tx {
        Ok(deposit_tx) => {
            let mut sp = Spinner::new(
                Spinners::from_str("Dots9").unwrap(),
                "Transaction submitted. Awaiting block confirmations...".into(),
            );
            let depsoit_receipt = deposit_tx.confirmations(1).await?.unwrap();  
            let deposit_msg = format!(
                "{}{}{}{}{}" ,
                String::from("\nTokens deposited in vault !!\n#################################\n✅ Hash : "),
                format!("0x{}",hex::encode(depsoit_receipt.transaction_hash.as_bytes().to_vec())), 
                String::from("\nVault Id : "),
                deposit_vault_id ,
                String::from("\n-----------------------------------\n")
            ) ; 
            sp.stop_with_message(deposit_msg.into());  
        }
        Err(_) => {
            println!("\n❌ Transaction Rejected.");
        }
    }

    Ok(())
}