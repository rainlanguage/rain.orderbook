
use std::str::FromStr;
use std::convert::TryFrom;
use clap::Parser; 
use ethers::providers::Middleware;
use ethers_signers::{Ledger, HDPath};
use ethers::{providers::{Provider, Http}, types::H160} ; 
use anyhow::anyhow;


use crate::orderbook::add_order::v3::add_ob_order;

use super::registry::RainNetworkOptions; 


#[derive(Parser,Debug,Clone)]
pub struct AddOrder{ 

    /// network to deposit
    #[arg(short, long)]
    pub network: RainNetworkOptions,  

    /// address of the orderbook 
    #[arg(long)]
    orderbook : String, 

    /// address of the token to deposit
    #[arg(short='p', long)]
    parser_address : String, 

    /// token list to be included in order
    #[arg(short,long,num_args = 1.. )]
    tokens : Vec<String>, 

    /// token list to be included in order
    #[arg(short,long,num_args = 1..)]
    decimals : Vec<u8>, 

    /// address of the token to deposit
    #[arg(short, long)]
    order_string : String, 

    /// address of the token to deposit
    #[arg(short='m', long)]
    order_meta : String, 

    /// address index of the wallet to accessed. defualt 0.
    #[arg(long, default_value="0")]
    address_index : Option<usize> , 
    
    /// mumbai rpc url, default read from env varibales
    #[arg(long,env)]
    pub mumbai_rpc_url: Option<String> , 

    /// polygon rpc url, default read from env varibales
    #[arg(long,env)]
    pub polygon_rpc_url: Option<String> ,

    /// ethereum rpc url, default read from env varibales
    #[arg(long,env)]
    pub ethereum_rpc_url: Option<String> ,  

    /// fuji rpc url, default read from env varibales
    #[arg(long,env)]
    pub fuji_rpc_url: Option<String> ,

    /// blocknative api key for gas oracle
    #[arg(long,env)]
    pub blocknative_api_key : Option<String> ,     

}  

impl AddOrder{
    pub fn get_network_rpc(&self) -> anyhow::Result<String>{
        let rpc_url = match self.network.clone()  {
            RainNetworkOptions::Ethereum => {
                if self.ethereum_rpc_url.is_none(){
                    return Err(anyhow!("\n ❌Please provide --ethereum-rpc-url argument.")) ;
                }
                self.ethereum_rpc_url.clone().unwrap()
            } ,
            RainNetworkOptions::Polygon => {
                if self.polygon_rpc_url.is_none(){
                    return Err(anyhow!("\n ❌Please provide --polygon-rpc-url argument.")) ;
                }
                self.polygon_rpc_url.clone().unwrap()
            },
            RainNetworkOptions::Mumbai => { 
                if self.mumbai_rpc_url.is_none(){
                    return Err(anyhow!("\n ❌Please provide --mumbai-rpc-url argument.")) ;
                }  
                self.mumbai_rpc_url.clone().unwrap()
            },
            RainNetworkOptions::Fuji => {
                if self.fuji_rpc_url.is_none(){
                    return Err(anyhow!("\n ❌Please provide --fuji-rpc-url argument.")) ;
                }
                self.fuji_rpc_url.clone().unwrap()
            }
        } ; 
        Ok(rpc_url)
    } 
}

pub async fn handle_add_order(add_order : AddOrder) -> anyhow::Result<()> {  

    let rpc_url = add_order.get_network_rpc().unwrap() ; 

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

    let parser_address = H160::from_str(&String::from(add_order.parser_address)).unwrap(); 
    let orderbook_address = H160::from_str(&String::from(add_order.orderbook)).unwrap();


    let _ = add_ob_order(
        orderbook_address,
        parser_address,
        add_order.tokens,
        add_order.decimals,
        add_order.order_string,
        add_order.order_meta,
        rpc_url,
        wallet,
        add_order.blocknative_api_key
    ).await ;

    Ok(())

} 
