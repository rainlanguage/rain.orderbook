
use std::str::FromStr;
use std::{convert::TryFrom, sync::Arc};
use clap::Parser; 
use ethers::prelude::k256::elliptic_curve::bigint::modular::constant_mod::ResidueParams;
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

pub async fn add_order(add_order : AddOrder) -> anyhow::Result<()> {  

    let wallet: LocalWallet = add_order.private_key.parse().unwrap()  ; 

    let rpc_url = add_order.get_network_rpc().unwrap() ;
    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?; 

    abigen!(IParserV1, "src/cli/abis/IParserV1.json");  

    abigen!(IOrderBookV2, "src/cli/abis/IOrderBookV2.json");  

    let parser_address = H160::from_str(&String::from(add_order.parser_address)).unwrap(); 
    let orderbook_address = H160::from_str(&String::from(add_order.orderbook)).unwrap(); ; 


    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(client.clone())); 

    let parser_contract = IParserV1::new(parser_address.clone(),Arc::new(client.clone())) ;   

    let (sources,constants) = parser_contract.parse(
        Bytes::from(
            add_order.order_string.as_bytes().to_vec()
        )
    ).call().await.unwrap() ; 

    let tokens = add_order.tokens ;
    let decimals = add_order.decimals ;

    let vault_id = U256::from(H160::random().as_bytes()) ;   
    let mut decimals = decimals.iter() ;

    let io_arr: Vec<_> = tokens.iter().map(|x| {
        Io {
            token : H160::from_str(x).unwrap() ,
            decimals : *decimals.next().unwrap(),
            vault_id : vault_id.clone()

        }
    }).collect() ; 

    let eval_config = EvaluableConfig {
        deployer : parser_address,
        sources : sources ,
        constants : constants
    } ;

    let rain_magic_number = String::from("ff0a89c674ee7874") ; 
 
    let meta_string = hex::decode(
        format!("{}{}",
        rain_magic_number,
        hex::encode(add_order.order_meta)
        )
    ).unwrap();

    let meta_bytes = Bytes::from(meta_string) ; 

    let order_config = OrderConfig {
        valid_inputs : io_arr.clone() ,
        valid_outputs : io_arr.clone(),
        evaluable_config : eval_config ,
        meta : meta_bytes
    } ; 

    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Adding order...".into(),
    ); 

    let order_tx = orderbook.add_order(order_config) ; 

    let order_pending_tx = order_tx.send().await?;

    let order_receipt = order_pending_tx.confirmations(3).await?.unwrap(); 

    let order_msg = format!(
        "{}{}{}" ,
        String::from("\nOrder added !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(order_receipt.transaction_hash.as_bytes().to_vec())), 
        String::from("\n-----------------------------------\n")
    ) ; 
    sp.stop_with_message(order_msg.into()); 


    Ok(())

} 
