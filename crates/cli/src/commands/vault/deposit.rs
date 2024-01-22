use crate::{
    execute::Execute,
    transaction::{CliTransactionCommandArgs, ExecuteTransaction},
};
use alloy_ethers_typecast::{ethers_address_to_alloy, transaction::WriteTransaction};
use alloy_primitives::U256;
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::IOrderBookV3::depositCall;
use rain_orderbook_common::{deposit::DepositArgs, transaction::TransactionArgs};
use tracing::info;

pub type Deposit = CliTransactionCommandArgs<CliDepositArgs>;

impl Execute for Deposit {
    async fn execute(&self) -> Result<()> {
        let tx_args: TransactionArgs = self.transaction_args.clone().into();
        let deposit_args: DepositArgs = self.cmd_args.clone().into();

        // ERC20 approve call
        info!("Step 1/2: Approve token transfer");
        let ledger_client = tx_args.clone().try_into_ledger_client().await?;
        let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
        let approve_call = deposit_args.clone().into_approve_call(ledger_address);
        let params = tx_args
            .try_into_write_contract_parameters(approve_call)
            .await?;
        WriteTransaction::new(ledger_client.client, params, 4, |status| {
            info!("Transaction status updated: {:?}", status);
        })
        .execute()
        .await?;

        // Orderbook deposit call
        info!("Step 2/2: Deposit tokens into vault");
        let ledger_client = tx_args.clone().try_into_ledger_client().await?;
        let deposit_call: depositCall = deposit_args.clone().try_into()?;
        let params = tx_args
            .try_into_write_contract_parameters(deposit_call)
            .await?;
        WriteTransaction::new(ledger_client.client, params, 4, |status| {
            info!("Transaction status updated: {:?}", status);
        })
        .execute()
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
