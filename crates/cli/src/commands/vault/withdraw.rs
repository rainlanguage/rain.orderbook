use crate::status::display_write_transaction_status;
use crate::{execute::Execute, transaction::CliTransactionCommandArgs};
use alloy_primitives::U256;
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::transaction::TransactionArgs;
use rain_orderbook_common::withdraw::WithdrawArgs;

pub type Withdraw = CliTransactionCommandArgs<CliWithdrawArgs>;

impl Execute for Withdraw {
    async fn execute(&self) -> Result<()> {
        let tx_args: TransactionArgs = self.transaction_args.clone().into();
        let withdraw_args: WithdrawArgs = self.cmd_args.clone().into();

        println!("----- Transaction (1/2): Withdraw tokens from Vault -----");
        withdraw_args
            .execute(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;
        Ok(())
    }
}

#[derive(Args, Clone)]
pub struct CliWithdrawArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: U256,

    #[arg(short = 'a', long, help = "The target amount to withdraw")]
    target_amount: U256,
}

impl From<CliWithdrawArgs> for WithdrawArgs {
    fn from(val: CliWithdrawArgs) -> Self {
        WithdrawArgs {
            token: val.token,
            vault_id: val.vault_id,
            target_amount: val.target_amount,
        }
    }
}
