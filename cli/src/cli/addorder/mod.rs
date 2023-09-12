
use std::str::FromStr;
use std::convert::TryFrom;
use clap::Parser; 
use ethers::providers::Middleware;
use ethers_signers::{Ledger, HDPath};
use ethers::{providers::{Provider, Http}, types::H160} ; 

use ethers::prelude::SignerMiddleware ; 
use crate::orderbook::add_order::v3::add_ob_order;
use spinners::{Spinner, Spinners};


#[derive(Parser,Debug,Clone)]
pub struct AddOrder{ 

    /// address of the orderbook 
    #[arg(long)]
    pub orderbook : String, 

    /// address of the token to deposit
    #[arg(short='p', long)]
    pub parser_address : String, 

    /// token list to be included in order
    #[arg(short,long,num_args = 1.. )]
    pub tokens : Vec<String>, 

    /// token list to be included in order
    #[arg(short,long,num_args = 1..)]
    pub decimals : Vec<u8>, 

    /// address of the token to deposit
    #[arg(short, long)]
    pub order_string : String, 

    /// address of the token to deposit
    #[arg(short='m', long)]
    pub order_meta : String, 

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

pub async fn handle_add_order(add_order : AddOrder) -> anyhow::Result<()> {  

    let rpc_url = add_order.rpc_url.unwrap() ; 

    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");  

    let chain_id = provider.get_chainid().await.unwrap().as_u64() ; 
    let wallet= Ledger::new(HDPath::Other(
        format!(
            "{}{}",
            String::from("m/44'/60'/0'/0/"),
            add_order.address_index.unwrap().to_string()
        )
    ), chain_id.clone()).await.expect("\n❌Could not instantiate Ledger Wallet");  

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;  
    let parser_address = H160::from_str(&String::from(add_order.parser_address)).unwrap(); 
    let orderbook_address = H160::from_str(&String::from(add_order.orderbook)).unwrap();


    let order_tx = add_ob_order(
        orderbook_address,
        parser_address,
        add_order.tokens,
        add_order.decimals,
        add_order.order_string,
        add_order.order_meta,
        rpc_url,
        add_order.blocknative_api_key
    ).await? ; 

    println!("\n-----------------------------------\nAdding order to Orderbook\n");
    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Awaiting confirmation from wallet...".into(),
    );  
    let order_tx = client.send_transaction(order_tx, None).await;   
    sp.stop() ;   

    match order_tx {
        Ok(order_tx) => {
            let mut sp = Spinner::new(
                Spinners::from_str("Dots9").unwrap(),
                "Transaction submitted. Awaiting block confirmations...".into(),
            );
            let order_receipt = order_tx.confirmations(1).await?.unwrap();  
            let order_msg = format!(
                "{}{}{}" ,
                String::from("\nOrder added !!\n#################################\n✅ Hash : "),
                format!("0x{}",hex::encode(order_receipt.transaction_hash.as_bytes().to_vec())), 
                String::from("\n-----------------------------------\n")
            ) ; 
            sp.stop_with_message(order_msg.into()); 
        }
        Err(_) => {
            println!("\n❌ Transaction Rejected.");
        }
    }

    Ok(())

} 
