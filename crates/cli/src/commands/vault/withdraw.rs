use crate::status::display_write_transaction_status;
use crate::{execute::Execute, transaction::CliTransactionArgs};
use alloy::primitives::{Address, U256};
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::transaction::TransactionArgs;
use rain_orderbook_common::withdraw::WithdrawArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultWithdrawArgs {
    #[arg(short = 'i', long, help = "The ID of the vault")]
    vault_id: U256,

    #[arg(short, long, help = "The token address in hex format")]
    token: Address,

    #[arg(short = 'a', long, help = "The target amount to withdraw")]
    target_amount: U256,

    #[clap(flatten)]
    transaction_args: CliTransactionArgs,
}

impl From<CliVaultWithdrawArgs> for WithdrawArgs {
    fn from(val: CliVaultWithdrawArgs) -> Self {
        WithdrawArgs {
            token: val.token,
            vault_id: val.vault_id,
            target_amount: val.target_amount,
        }
    }
}

impl Execute for CliVaultWithdrawArgs {
    async fn execute(&self) -> Result<()> {
        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;
        let withdraw_args: WithdrawArgs = self.clone().into();

        info!("----- Withdraw tokens from Vault -----");
        withdraw_args
            .execute(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;
        Ok(())
    }
}
