use crate::{
    execute::Execute, status::display_write_transaction_status,
    transaction::CliTransactionCommandArgs,
};
use alloy_primitives::U256;
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::{deposit::DepositArgs, transaction::TransactionArgs};

pub type Deposit = CliTransactionCommandArgs<CliDepositArgs>;

impl Execute for Deposit {
    async fn execute(&self) -> Result<()> {
        let tx_args: TransactionArgs = self.transaction_args.clone().into();
        let deposit_args: DepositArgs = self.cmd_args.clone().into();

        println!("----- Transaction (1/2): Approve ERC20 token spend -----");
        deposit_args
            .execute(
                tx_args,
                |status| {
                    display_write_transaction_status(status);
                },
                |status| {
                    display_write_transaction_status(status);
                },
                || {
                    println!("----- Transaction (2/2): Deposit tokens into Orderbook -----");
                },
            )
            .await?;
        Ok(())
    }
}

#[derive(Args, Clone)]
pub struct CliDepositArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: U256,

    #[arg(short, long, help = "The amount to deposit")]
    amount: U256,
}

impl From<CliDepositArgs> for DepositArgs {
    fn from(val: CliDepositArgs) -> Self {
        DepositArgs {
            token: val.token,
            vault_id: val.vault_id,
            amount: val.amount,
        }
    }
}
