use anyhow::Result;
use clap::command;
use clap::{Parser, Subcommand};

pub mod registry;
pub mod deposit;
pub mod withdraw;
pub mod addorder;
pub mod removeorder;



#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    orderbook: Orderbook,
} 



#[derive(Subcommand)]
pub enum Orderbook {
    Deposit(deposit::Deposit),
    Withdraw(withdraw::Withdraw),
    AddOrder(addorder::AddOrder),
    RemoveOrder(removeorder::RemoveOrder),
    ListOrders
}

pub async fn dispatch(orderbook: Orderbook) -> Result<()> {
    match orderbook {
        Orderbook::Deposit(deposit) => {
            let _ = deposit::handle_deposit(deposit).await ; 
            Ok(())
        },
        Orderbook::Withdraw(withdraw) => {
            let _ = withdraw::handle_withdraw(withdraw).await;
            Ok(())
        },
        Orderbook::AddOrder(order) => {
            let _ = addorder::handle_add_order(order).await;
            Ok(())
        },
        Orderbook::RemoveOrder(order) => {
            let _ = removeorder::handle_remove_order(order).await;
            Ok(())
        } ,
        Orderbook::ListOrders => {
            let _ = crate::subgraph::showorder::query().await ;
            Ok(())
        }
    }
}

pub async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;

    let cli = Cli::parse();
    dispatch(cli.orderbook).await
}
