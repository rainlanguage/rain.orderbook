
use clap::Parser;
use self::listorders::view_orders;

pub mod listorders;

#[derive(Parser,Debug,Clone)]
pub struct ListOrder{
    /// network to deposit
    #[arg(short, long)]
    pub subgraph_url: String,  
}  

pub async fn handle_list_order(list_order : ListOrder) -> anyhow::Result<()> { 
    view_orders(list_order.subgraph_url.clone()).await ;
    Ok(())
} 