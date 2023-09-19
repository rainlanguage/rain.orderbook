
use std::str::FromStr;
use std::convert::TryFrom;
use clap::Parser; 
use ethers::{providers::Middleware, types::U256};
use ethers_signers::{Ledger, HDPath};
use ethers::{providers::{Provider, Http}, types::H160} ; 
use tracing::{error, info}; 
use crate::{orderbook::add_order::v3::add_ob_order, transaction::execute_transaction};
use anyhow::anyhow;

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

    /// associated vault id
    #[arg(short, long)]
    pub vault_id : String ,

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

    let rpc_url = match add_order.rpc_url {
        Some(url) => {
            url
        },
        None => {
            error!("RPC URL NOT PROVIDED") ; 
            return Err(anyhow!("RPC URL not provided.")) ;
        }
    } ;  

    let vault_id = match U256::from_dec_str(&String::from(add_order.vault_id)){
        Ok(id) => id ,
        Err(err) => {
            error!("INVALID VAULT ID: {}",err) ; 
            return Err(anyhow!(err)) ;
        }
    };

    let provider = match Provider::<Http>::try_from(rpc_url.clone()){
        Ok(provider) => {
            provider
        },
        Err(err) => {
            error!("INVALID RPC URL: {}",err) ; 
            return Err(anyhow!(err)) ;
        }
    } ;  

    let chain_id = provider.get_chainid().await.unwrap().as_u64() ; 
    let wallet= match Ledger::new(HDPath::Other(
        format!(
            "{}{}",
            String::from("m/44'/60'/0'/0/"),
            add_order.address_index.unwrap().to_string()
        )
    ), chain_id.clone()).await {
        Ok(wallet) => {
            wallet
        },
        Err(err) => {
            error!("ERROR INSTANTIATING LEDGER WALLET: {}",err) ; 
            return Err(anyhow!(err)) ;
        }
    } ; 

    let parser_address = H160::from_str(&String::from(add_order.parser_address)).unwrap(); 
    let orderbook_address = H160::from_str(&String::from(add_order.orderbook)).unwrap(); 

    let tokens = add_order.tokens.iter().map(|t| {
        H160::from_str(&String::from(t)).unwrap()
    }).collect::<Vec<H160>>() ;


    let order_tx = add_ob_order(
        orderbook_address,
        parser_address,
        tokens,
        add_order.decimals,
        vault_id,
        add_order.order_string,
        add_order.order_meta,
        rpc_url.clone(),
        add_order.blocknative_api_key
    ).await? ; 

    info!("Adding order to Orderbook");
    let _ = execute_transaction(rpc_url.clone(),wallet,order_tx).await? ;
    Ok(())

} 
