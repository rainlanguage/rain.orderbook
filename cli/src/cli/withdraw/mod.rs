
use std::str::FromStr;
use std::convert::TryFrom;
use clap::Parser; 
use ethers::types::Address;
use ethers::utils::parse_units;
use ethers::{providers::{Provider, Middleware, Http}, types::U256} ; 
use anyhow::anyhow;
use ethers_signers::{Ledger, HDPath};
use crate::cli::withdraw::withdraw::withdraw_tokens;
use super::registry::RainNetworkOptions;

pub mod withdraw ;

#[derive(Parser,Debug,Clone)]
pub struct Withdraw{ 

    /// network to withdraw
    #[arg(short, long)]
    pub network: RainNetworkOptions,  

    /// address of the orderbook 
    #[arg(short, long)]
    orderbook : String, 

    /// address of the token to withdraw
    #[arg(long)]
    token_address : String, 

    /// decimals coressponding to the token
    #[arg( long)]
    token_decimals : u32, 

    /// amount to withdraw.
    #[arg(long)]
    amount : String,

    /// decimal vault id to withdraw from
    #[arg(long)]
    vault_id : String , 

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

}  

impl Withdraw{
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
       
    let rpc_url = withdraw.get_network_rpc().unwrap() ;

    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");  

    let chain_id = provider.get_chainid().await.unwrap().as_u64() ; 
    let wallet= Ledger::new(HDPath::LedgerLive(0), chain_id).await?;

    let _ = withdraw_tokens(
        token_address,
        token_amount,
        vault_id,
        orderbook_address,
        rpc_url,
        wallet
    ).await;
    
    Ok(())
}