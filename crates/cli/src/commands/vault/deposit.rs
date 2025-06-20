use crate::{
    execute::Execute, status::display_write_transaction_status, transaction::CliTransactionArgs,
};
use alloy::primitives::{Address, B256, U256};
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::{deposit::DepositArgs, erc20::ERC20, transaction::TransactionArgs};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultDepositArgs {
    #[arg(short = 'i', long, help = "The ID of the vault")]
    vault_id: B256,

    #[arg(short, long, help = "The token address in hex format")]
    token: Address,

    #[arg(short, long, help = "The amount to deposit")]
    amount: U256,

    #[arg(short, long, help = "The decimals of the token")]
    decimals: Option<u8>,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

impl Execute for CliVaultDepositArgs {
    async fn execute(&self) -> Result<()> {
        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;

        let decimals = if let Some(decimals) = self.decimals {
            decimals
        } else {
            let rpc_url: url::Url = tx_args.rpc_url.parse()?;
            let erc20 = ERC20::new(rpc_url, self.token);
            erc20.decimals().await?
        };

        let deposit_args: DepositArgs = DepositArgs {
            token: self.token,
            vault_id: self.vault_id,
            amount: self.amount,
            decimals,
        };

        info!("----- Approve ERC20 token spend -----");
        deposit_args
            .execute_approve(tx_args.clone(), |status| {
                display_write_transaction_status(status);
            })
            .await?;

        info!("----- Deposit tokens into Orderbook -----");
        deposit_args
            .execute_deposit(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;
        Ok(())
    }
}
