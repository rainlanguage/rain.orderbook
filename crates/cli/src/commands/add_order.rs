use crate::execute::{ExecutableTransactionCall, Execute};
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::IOrderBookV3::addOrderCall;
use rain_orderbook_common::add_order::AddOrderArgs;
use std::path::PathBuf;

pub type AddOrder = ExecutableTransactionCall<CliAddOrderArgs>;

impl Execute for AddOrder {
    async fn execute(self) -> Result<()> {
        let add_order_args: AddOrderArgs = self.call_args.clone().into();
        let add_order_call: addOrderCall = add_order_args.try_into()?;

        self.execute_transaction_call(add_order_call).await
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

    #[arg(short = 'a', long, help = "Deployer address")]
    deployer: String,
}

impl From<CliAddOrderArgs> for AddOrderArgs {
    fn from(val: CliAddOrderArgs) -> Self {
        Self {
            dotrain_path: val.dotrain_path,
            deployer: val.deployer,
        }
    }
}
