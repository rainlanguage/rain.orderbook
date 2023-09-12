
use std::str::FromStr;
use std::convert::TryFrom;
use clap::Parser; 
use ethers::types::Address;
use ethers::utils::parse_units;
use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256}};
use anyhow::anyhow;

use ethers_signers::{Ledger, HDPath};
use crate::orderbook::deposit::v3::deposit_token; 
use spinners::{Spinner, Spinners};  
use ethers::prelude::SignerMiddleware ; 

use crate::tokens::approve_tokens;

use serde::Deserialize;
#[derive(Parser,Debug,Clone,Deserialize)]
pub struct Deposit{ 

    /// address of the orderbook 
    #[arg(short, long)]
    pub orderbook : String, 

    /// address of the token to deposit
    #[arg(short='t', long, num_args = 1..)]
    pub token_address : String, 

    /// decimals coressponding to the token
    #[arg(short='d', long, num_args = 1..)]
    pub token_decimals : u32, 

    /// amount to deposit.
    #[arg(short, long)]
    pub amount : String,

    /// optional vault id to deposit in (in decimals)
    #[arg(short, long)]
    pub vault_id : Option<String> ,

    /// address index of the wallet to accessed. defualt 0.
    #[arg(long, default_value="0")]
    pub address_index : Option<usize> , 

    /// rpc url, default read from env varibales
    #[arg(long,env)]
    pub rpc_url: Option<String> , 

    /// blocknative api key for gas oracle
    #[arg(long,env)]
    pub blocknative_api_key : Option<String> ,    

} 


pub async fn handle_deposit(deposit : Deposit) -> anyhow::Result<()> { 

    let orderbook_address = match Address::from_str(&deposit.orderbook) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(anyhow!("\n❌Incorrect orderbook address.")) ;
        }
    };

    let token_address = match Address::from_str(&deposit.token_address) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(anyhow!("\n❌Incorrect token address.")) ;
        }
    };  

    let token_amount: U256 = match parse_units(deposit.amount.clone(),deposit.token_decimals.clone()) {
        Ok(amount) => amount.into() ,
        Err(_) => {
            return Err(anyhow!("\n❌Incorrect amount.")) ;
        }
    } ;

    let vault_id = match deposit.vault_id.clone() {
        Some(val) => {
            match U256::from_dec_str(&val) {
                Ok(id) => id ,
                Err(_) => {
                    return Err(anyhow!("\n❌Invalid vault id.")) ;
                }
            }
        } ,
        None => {
            U256::from(H160::random().as_bytes()) 
        }
    } ;  

    let rpc_url = deposit.rpc_url.unwrap() ; 
    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");  

    let chain_id = provider.get_chainid().await.unwrap().as_u64() ; 
    let wallet= Ledger::new(HDPath::Other(
        format!(
            "{}{}",
            String::from("m/44'/60'/0'/0/"),
            deposit.address_index.unwrap().to_string()
        )
    ), chain_id.clone()).await.expect("\n❌Could not instantiate Ledger Wallet");  

    let wallet_address =  wallet.get_address().await.unwrap() ; 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;     

    // Approve token for deposit 
    let approve_tx = approve_tokens(
        token_address.clone() ,
        token_amount.clone(),
        orderbook_address.clone() ,
        rpc_url.clone(),
        wallet_address,
        deposit.blocknative_api_key.clone()
    ).await? ;  
    
    // Approve Tokens For deposit
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
    // Tokens approved. 

    // Deposit tokens
    let deposit_tx = deposit_token(
        token_address,
        token_amount,
        vault_id.clone(),
        orderbook_address,
        rpc_url,
        deposit.blocknative_api_key
    ).await? ; 

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
                vault_id ,
                String::from("\n-----------------------------------\n")
            ) ; 
            sp.stop_with_message(deposit_msg.into());  
        }
        Err(_) => {
            println!("\n❌ Transaction Rejected.");
        }
    }
    // Tokens Deposited.
    Ok(())
}