
use std::str::FromStr;
use std::{convert::TryFrom, sync::Arc};
use clap::Parser; 
use ethers::types::Address;
use ethers::utils::{parse_units, Units};
use ethers::{providers::{Provider, Middleware, Http}, types::{H160, H256, U256,Bytes}} ; 
use ethers::{signers::LocalWallet, types::{Eip1559TransactionRequest, U64}, prelude::SignerMiddleware};
use anyhow::{Result, anyhow};
use ethers::core::abi::Abi ;
use ethers::contract::Contract;
use ethers::contract::abigen ;
use ethers::contract::Abigen ;
use spinners::{Spinner, Spinners};

use super::registry::RainNetworkOptions;

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

    /// vault id to withdraw from
    #[arg(long)]
    vault_id : String , 

    /// private key (unprefixed) provided when deploy is set to true
    #[arg(short ='k' , long = "priavte-key" )]
    pub private_key: String, 

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

pub async fn withdraw(withdraw : Withdraw) -> anyhow::Result<()> { 

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

    // let vault_id = match  {
    //     Ok(id) => id ,
    //     Err(_) => {
    //         return Err(anyhow!("\n ❌Invalid vault id.")) ;
    //     }
    // } ; 
    
    // let i_p = ethers::utils::hex::((withdraw.clone().vault_id)) ;
    // // let i_p = U256::from_big_endian(&i_p) ;
    // println!("i_p : {:?}",i_p) ;

    let vault_id = U256::from_str(&String::from(withdraw.clone().vault_id)).unwrap();
       
    let rpc_url = withdraw.get_network_rpc().unwrap() ;

    let wallet: LocalWallet = withdraw.private_key.parse().unwrap() ;


    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?; 
    let signer_address =  client.address() ;

    abigen!(IOrderBookV2, "src/cli/abis/IOrderBookV2.json"); 

    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(client));  

    let vault_balance: U256 = orderbook.vault_balance(signer_address, token_address, vault_id).call().await.unwrap() ; 


    if(token_amount.gt(&vault_balance)){ 

        let err_msg = format!(
            "{}{}" ,
            String::from("\n#################################\nInsufficient vault balance for withdrawal.\nCurrent vault balance :"),
            vault_balance.to_string()
        ) ;  
        println!("{}",err_msg) ;
        return Err(anyhow!(err_msg)); 
    }

    let withdraw_config = WithdrawConfig{
        token : token_address ,
        vault_id : vault_id ,
        amount : token_amount
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