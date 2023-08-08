
use clap::Parser;
use self::listorders::list_orders;  

pub mod listorders;

#[derive(Parser,Debug,Clone)]
pub struct ListOrder{
    /// network to deposit
    #[arg(short, long)]
    pub subgraph_url: String,  
}  

pub async fn handle_list_order(list_order : ListOrder) -> anyhow::Result<()> { 
     let _= list_orders(list_order.subgraph_url.clone()).await ;
    Ok(())
} 