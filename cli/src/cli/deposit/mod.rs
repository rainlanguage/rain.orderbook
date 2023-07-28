
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
pub struct Deposit{ 

    /// network to deposit
    #[arg(short, long)]
    pub network: RainNetworkOptions,  

    /// address of the orderbook 
    #[arg(short, long)]
    orderbook : String, 

    /// address of the token to deposit
    #[arg(short='t', long)]
    token_address : String, 

    /// decimals coressponding to the token
    #[arg(short='d', long)]
    token_decimals : u32, 

    /// amount to deposit.
    #[arg(short, long)]
    amount : String,

    /// optional vault id to deposit in
    #[arg(short, long)]
    vault_id : Option<String> , 

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

impl Deposit{
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


pub async fn deposit(deposit : Deposit) -> anyhow::Result<()> { 

    let orderbook_address = match Address::from_str(&deposit.orderbook) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect orderbook address.")) ;
        }
    };

    let token_address = match Address::from_str(&deposit.token_address) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect token address.")) ;
        }
    };  

    let token_amount: U256 = match parse_units(deposit.amount.clone(),deposit.token_decimals.clone()) {
        Ok(amount) => amount.into() ,
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect amount.")) ;
        }
    } ;

    let vault_id = match deposit.vault_id.clone() {
        Some(val) => {
            match U256::from_str(&val) {
                Ok(id) => id ,
                Err(_) => {
                    return Err(anyhow!("\n ❌Invalid vault id.")) ;
                }
            }
        } ,
        None => {
            U256::from(H160::random().as_bytes()) 
        }
    } ; 

    let rpc_url = deposit.get_network_rpc().unwrap() ;

    let wallet: LocalWallet = deposit.private_key.parse().unwrap() ;


    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?; 
    let signer_address =  client.address() ;

    abigen!(IOrderBookV2, "src/cli/abis/IOrderBookV2.json"); 

    abigen!(
        IERC20,
        r#"[
            function totalSupply() external view returns (uint256)
            function balanceOf(address account) external view returns (uint256)
            function transfer(address recipient, uint256 amount) external returns (bool)
            function allowance(address owner, address spender) external view returns (uint256)
            function approve(address spender, uint256 amount) external returns (bool)
            function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
            event Transfer(address indexed from, address indexed to, uint256 value)
            event Approval(address indexed owner, address indexed spender, uint256 value)
        ]"#,
    ); 

    let token_contract = IERC20::new(token_address,Arc::new(client.clone())) ; 
    let token_balance: U256 = token_contract.balance_of(signer_address.clone()).call().await.unwrap() ;  

    if token_balance.gt(&token_amount.clone()) {
        let approve_tx = token_contract.approve(orderbook_address.clone(), token_amount.clone()) ; 
        let mut sp = Spinner::new(
            Spinners::from_str("Dots9").unwrap(),
            "Approving tokens for deposit...".into(),
        );  
        let approve_pending_tx = approve_tx.send().await? ;
        let approve_receipt = approve_pending_tx.confirmations(4).await?.unwrap();  

        let end_msg = format!(
            "{}{}{}" ,
            String::from("\nTokens Approved for deposit !!\n#################################\n✅ Hash : "),
            format!("0x{}",hex::encode(approve_receipt.transaction_hash.as_bytes().to_vec())), 
            String::from("\n-----------------------------------\n")
        ) ; 
        sp.stop_with_message(end_msg.into()); 


    }else{
        return Err(anyhow!("\n ❌Insufficent balance for deposit.\nCurrent Balance : {}.",token_balance)) ;
    } 

    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(client)); 

    let deposit_config = DepositConfig{
        token : token_address ,
        vault_id : vault_id ,
        amount : token_amount
    } ; 

    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Depositing token in vault...".into(),
    ); 

    let deposit_tx = orderbook.deposit(deposit_config) ;
    let deposit_pending_tx = deposit_tx.send().await?;
    let depsoit_receipt = deposit_pending_tx.confirmations(3).await?.unwrap(); 

    let deposit_msg = format!(
        "{}{}{}" ,
        String::from("\nTokens deposited in vault !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(depsoit_receipt.transaction_hash.as_bytes().to_vec())), 
        String::from("\n-----------------------------------\n")
    ) ; 
    sp.stop_with_message(deposit_msg.into()); 
    
    Ok(())
}