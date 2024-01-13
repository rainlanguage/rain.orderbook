use crate::execute::{ExecutableTransactionCall, Execute};
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::IOrderBookV3::withdrawCall;
use rain_orderbook_common::withdraw::WithdrawArgs;

pub type Withdraw = ExecutableTransactionCall<CliWithdrawArgs>;

impl Execute for Withdraw {
    async fn execute(self) -> Result<()> {
        let withdraw_args: WithdrawArgs = self.call_args.clone().into();
        let withdraw_call: withdrawCall = withdraw_args.try_into()?;

        self.execute_transaction_call(withdraw_call).await
    }
}

#[derive(Args, Clone)]
pub struct CliWithdrawArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,

    #[arg(short = 'a', long, help = "The target amount to withdraw")]
    target_amount: u64,
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
