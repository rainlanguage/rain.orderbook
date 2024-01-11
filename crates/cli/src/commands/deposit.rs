use crate::call::{ExecutableTransactionCall, Execute};
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::IOrderBookV3::depositCall;
use rain_orderbook_common::deposit::DepositArgs;

pub type Deposit = ExecutableTransactionCall<CliDepositArgs>;

impl Execute for Deposit {
    async fn execute(self) -> Result<()> {
        let deposit_args: DepositArgs = self.call_args.clone().into();
        let deposit_call: depositCall = deposit_args.try_into()?;

        self.execute_transaction_call(deposit_call).await
    }
}

#[derive(Args, Clone)]
pub struct CliDepositArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,

    #[arg(short, long, help = "The amount to deposit")]
    amount: u64,
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
