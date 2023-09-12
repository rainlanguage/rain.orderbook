use clap::Parser;
use ethers::{providers::{Provider, Middleware, Http}, types::H160};
use crate::{subgraph::remove_order::v3::get_remove_order, orderbook::remove_order::v3::remove_order} ;
use ethers_signers::{Ledger, HDPath};
use std::str::FromStr;
use ethers::prelude::SignerMiddleware ; 
use spinners::{Spinner, Spinners};
#[derive(Parser,Debug,Clone)]
pub struct RemoveOrder{ 

    /// address of the orderbook 
    #[arg(short, long)]
    pub orderbook : String, 

    /// address of the orderbook 
    #[arg(short, long)]
    pub subgraph_url : String, 

    /// id of the order to remove
    #[arg(short='i', long)]
    pub order_id : String, 

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


pub async fn handle_remove_order(order: RemoveOrder) -> anyhow::Result<()> {  

    let rpc_url = order.rpc_url.unwrap() ; 
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

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;     

    let order_to_remove = get_remove_order(order.subgraph_url, order.order_id).await.unwrap() ; 

    let remove_order_tx =  remove_order(
        order_to_remove,
        orderbook_address,
        rpc_url,
        order.blocknative_api_key
    ).await? ; 

    println!("\n-----------------------------------\nRemoving Orders\n");
    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Awaiting confirmation from wallet...".into(),
    );  
    let remove_order_tx = client.send_transaction(remove_order_tx, None).await;   
    sp.stop() ;

    match remove_order_tx {
        Ok(remove_order_tx) => {
            let mut sp = Spinner::new(
                Spinners::from_str("Dots9").unwrap(),
                "Transaction submitted. Awaiting block confirmations...".into(),
            );
            let remove_order_receipt = remove_order_tx.confirmations(1).await?.unwrap();  
            let order_msg = format!(
                "{}{}{}" ,
                String::from("\nOrder Removed !!\n#################################\n✅ Hash : "),
                format!("0x{}",hex::encode(remove_order_receipt.transaction_hash.as_bytes().to_vec())), 
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

