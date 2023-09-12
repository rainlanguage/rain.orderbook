
use std::str::FromStr;
use std::convert::TryFrom;
use clap::Parser; 
use ethers::types::Address;
use ethers::utils::parse_units;
use ethers::{providers::{Provider, Middleware, Http}, types::U256} ; 
use ethers::prelude::SignerMiddleware ;
use anyhow::anyhow;
use ethers_signers::{Ledger, HDPath};
use crate::orderbook::withdraw::v3::withdraw_tokens;
use spinners::{Spinner, Spinners};

#[derive(Parser,Debug,Clone)]
pub struct Withdraw{ 

    /// address of the orderbook 
    #[arg(short, long)]
    pub orderbook : String, 

    /// address of the token to withdraw
    #[arg(long)]
    pub token_address : String, 

    /// decimals coressponding to the token
    #[arg( long)]
    pub token_decimals : u32, 

    /// amount to withdraw.
    #[arg(long)]
    pub amount : String,

    /// decimal vault id to withdraw from
    #[arg(long)]
    pub vault_id : String , 

    /// address index of the wallet to accessed. defualt 0.
    #[arg(long, default_value="0")]
    pub address_index : Option<usize> , 

    /// mumbai rpc url, default read from env varibales
    #[arg(long,env)]
    pub rpc_url: Option<String> , 

    /// blocknative api key for gas oracle
    #[arg(long,env)]
    pub blocknative_api_key : Option<String> ,     

}  

pub async fn handle_withdraw(withdraw : Withdraw) -> anyhow::Result<()> { 

    let orderbook_address = match Address::from_str(&withdraw.orderbook) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect orderbook address.")) ;
        }
    };

    let token_address = match Address::from_str(&withdraw.token_address) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect token address.")) ;
        }
    };  

    let token_amount: U256 = match parse_units(withdraw.amount.clone(),withdraw.token_decimals.clone()) {
        Ok(amount) => amount.into() ,
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect amount.")) ;
        }
    } ;

    let vault_id = U256::from_dec_str(&String::from(withdraw.clone().vault_id)).unwrap();
       
    let rpc_url = withdraw.rpc_url.unwrap() ;

    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");  

    let chain_id = provider.get_chainid().await.unwrap().as_u64() ; 
    let wallet= Ledger::new(HDPath::Other(
        format!(
            "{}{}",
            String::from("m/44'/60'/0'/0/"),
            withdraw.address_index.unwrap().to_string()
        )
    ), chain_id.clone()).await.expect("\n❌Could not instantiate Ledger Wallet"); 

    let wallet_address =  wallet.get_address().await.unwrap() ; 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;     
 

    let withdraw_tx = withdraw_tokens(
        token_address,
        token_amount,
        vault_id,
        orderbook_address,
        rpc_url,
        wallet_address,
        withdraw.blocknative_api_key
    ).await?; 

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