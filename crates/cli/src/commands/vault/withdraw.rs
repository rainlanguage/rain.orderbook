use crate::{
    execute::Execute,
    transaction::{CliTransactionCommandArgs, ExecuteTransaction},
};
use alloy_primitives::U256;
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::IOrderBookV3::withdrawCall;
use rain_orderbook_common::withdraw::WithdrawArgs;

pub type Withdraw = CliTransactionCommandArgs<CliWithdrawArgs>;

impl Execute for Withdraw {
    async fn execute(&self) -> Result<()> {
        let mut execute_tx: ExecuteTransaction = self.clone().into();
        let withdraw_args: WithdrawArgs = self.cmd_args.clone().into();
        let withdraw_call: withdrawCall = withdraw_args.try_into()?;

        let ledger_client = execute_tx.connect_ledger().await?;
        execute_tx.send(ledger_client, withdraw_call).await
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
