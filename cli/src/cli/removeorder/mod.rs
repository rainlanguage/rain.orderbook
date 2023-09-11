use clap::Parser;
use ethers::{providers::{Provider, Middleware, Http}, types::H160} ; 
use crate::{cli::registry::RainNetworkOptions, subgraph::remove_order::v3::get_remove_order, orderbook::remove_order::v3::remove_order} ;
use anyhow::anyhow;
use ethers_signers::{Ledger, HDPath};
use std::str::FromStr;

#[derive(Parser,Debug,Clone)]
pub struct RemoveOrder{ 
    /// network to remove order from
    #[arg(short, long)]
    pub network: RainNetworkOptions,  

    /// address of the orderbook 
    #[arg(short, long)]
    orderbook : String, 

    /// address of the orderbook 
    #[arg(short, long)]
    subgraph_url : String, 

    /// id of the order to remove
    #[arg(short='i', long)]
    order_id : String, 

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

impl RemoveOrder{
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

pub async fn handle_remove_order(order: RemoveOrder) -> anyhow::Result<()> {  

    let rpc_url = order.get_network_rpc().unwrap() ; 
    let orderbook_address = H160::from_str(&String::from(order.orderbook)).unwrap();

    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider") ; 

    let chain_id = provider.get_chainid().await.unwrap().as_u64() ; 
    let wallet= Ledger::new(HDPath::Other(
        format!(
            "{}{}",
            String::from("m/44'/60'/0'/0/"),
            order.address_index.unwrap().to_string()
        )
    ), chain_id.clone()).await.expect("\n❌Could not instantiate Ledger Wallet");  

    let order_to_remove = get_remove_order(order.subgraph_url, order.order_id).await.unwrap() ;
    let _ =  remove_order(
        order_to_remove,
        orderbook_address,
        rpc_url,
        wallet,
        order.blocknative_api_key
    ).await ;

    Ok(())
}

