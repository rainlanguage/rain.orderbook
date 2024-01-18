use crate::{
    execute::Execute,
    transaction::{CliTransactionCommandArgs, ExecuteTransaction},
};
use alloy_ethers_typecast::ethers_address_to_alloy;
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::{IOrderBookV3::depositCall, IERC20::approveCall};
use rain_orderbook_common::deposit::DepositArgs;
use tracing::info;

pub type Deposit = CliTransactionCommandArgs<CliDepositArgs>;

impl Execute for Deposit {
    async fn execute(&self) -> Result<()> {
        // Prepare deposit call
        let deposit_args: DepositArgs = self.cmd_args.clone().into();
        let deposit_call: depositCall = deposit_args.clone().try_into()?;

        // Prepare approve call
        let mut execute_tx: ExecuteTransaction = self.clone().into();
        let ledger_client = execute_tx.connect_ledger().await?;
        let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
        let approve_call: approveCall =
            deposit_args.clone().try_into_approve_call(ledger_address)?;

        info!("Step 1/2: Approve token transfer");
        execute_tx.send(ledger_client, approve_call).await?;

        info!("Step 2/2: Deposit tokens into vault");
        let ledger_client = execute_tx.connect_ledger().await?;
        execute_tx.send(ledger_client, deposit_call).await
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
