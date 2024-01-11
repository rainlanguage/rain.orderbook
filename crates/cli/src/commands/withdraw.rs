use crate::transaction::CliTransactionArgs;
use alloy_ethers_typecast::transaction::ExecutableTransaction;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use rain_orderbook_bindings::IOrderBookV3::withdrawCall;
use rain_orderbook_common::{transaction::TransactionArgs, withdraw::WithdrawArgs};

#[derive(Parser)]
pub struct Withdraw {
    #[clap(flatten)]
    withdraw_args: CliWithdrawArgs,
    #[clap(flatten)]
    transaction_args: CliTransactionArgs,
}

impl Withdraw {
    pub async fn execute(self) -> Result<()> {
        let withdraw_args: WithdrawArgs = self.withdraw_args.into();
        let withdraw_call: withdrawCall = withdraw_args.try_into()?;
        let tx_args: TransactionArgs = self.transaction_args.into();
        let request = tx_args
            .to_transaction_request_with_call(withdraw_call)
            .await?;
        let ledger_client = tx_args.to_ledger_client().await?;

        let tx =
            ExecutableTransaction::from_alloy_transaction_request(request, ledger_client.client)
                .await?;

        tx.execute().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}

#[derive(Args)]
pub struct CliWithdrawArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,

    #[arg(short = 'a', long, help = "The target amount to withdraw")]
    target_amount: u64,
}

impl Into<WithdrawArgs> for CliWithdrawArgs {
    fn into(self) -> WithdrawArgs {
        WithdrawArgs {
            token: self.token,
            vault_id: self.vault_id,
            target_amount: self.target_amount,
        }
    }
}
