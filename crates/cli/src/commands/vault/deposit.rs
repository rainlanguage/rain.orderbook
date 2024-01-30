use crate::{
    execute::Execute, status::display_write_transaction_status, transaction::CliTransactionArgs,
};
use alloy_primitives::{Address, U256};
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::{deposit::DepositArgs, transaction::TransactionArgs};

#[derive(Args, Clone)]
pub struct CliVaultDepositArgs {
    #[arg(short='i', long, help = "The ID of the vault")]
    vault_id: U256,

    #[arg(short, long, help = "The token address in hex format")]
    token: Address,

    #[arg(short, long, help = "The amount to deposit")]
    amount: U256,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

impl From<CliVaultDepositArgs> for DepositArgs {
    fn from(val: CliVaultDepositArgs) -> Self {
        DepositArgs {
            token: val.token,
            vault_id: val.vault_id,
            amount: val.amount,
        }
    }
}

impl Execute for CliVaultDepositArgs {
    async fn execute(&self) -> Result<()> {
        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;
        let deposit_args: DepositArgs = self.clone().into();

        println!("----- Transaction (1/2): Approve ERC20 token spend -----");
        deposit_args
            .execute_approve(tx_args.clone(), |status| {
                display_write_transaction_status(status);
            })
            .await?;

        println!("----- Transaction (2/2): Deposit tokens into Orderbook -----");
        deposit_args
            .execute_deposit(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;
        Ok(())
    }
}
