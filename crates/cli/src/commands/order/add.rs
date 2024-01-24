use crate::{
    execute::Execute,
    transaction::{CliTransactionCommandArgs, ExecuteTransaction},
};
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::IOrderBookV3::addOrderCall;
use rain_orderbook_common::add_order::AddOrderArgs;
use std::fs::File;
use std::path::PathBuf;

pub type AddOrder = CliTransactionCommandArgs<CliAddOrderArgs>;

impl Execute for AddOrder {
    async fn execute(&self) -> Result<()> {
        let add_order_args: AddOrderArgs = self.cmd_args.clone().try_into()?;
        let add_order_call: addOrderCall = add_order_args.try_into()?;
        let mut execute_tx: ExecuteTransaction = self.clone().into();

        let ledger_client = execute_tx.connect_ledger().await?;
        execute_tx.send(ledger_client, add_order_call).await
    }
}

#[derive(Args, Clone)]
pub struct CliAddOrderArgs {
    #[arg(
        short = 'p',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_path: PathBuf,
}

impl TryFrom<CliAddOrderArgs> for AddOrderArgs {
    fn try_from(val: CliAddOrderArgs) -> Result<Self> {
        let mut file = File::open(val.dotrain_path)?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;

        Self { dotrain: text }
    }
}
