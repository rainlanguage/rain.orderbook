use crate::transaction::CliTransactionArgs;
use alloy_ethers_typecast::transaction::ExecutableTransaction;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use rain_orderbook_bindings::IOrderBookV3::depositCall;
use rain_orderbook_common::{deposit::DepositArgs, transaction::TransactionArgs};

#[derive(Parser)]
pub struct Deposit {
    #[clap(flatten)]
    deposit_args: CliDepositArgs,
    #[clap(flatten)]
    transaction_args: CliTransactionArgs,
}

impl Deposit {
    pub async fn execute(self) -> Result<()> {
        let deposit_args: DepositArgs = self.deposit_args.into();
        let deposit_call: depositCall = deposit_args.try_into()?;
        let tx_args: TransactionArgs = self.transaction_args.into();
        let request = tx_args
            .to_transaction_request_with_call(deposit_call)
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
pub struct CliDepositArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The amount to deposit")]
    amount: u64,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,
}

impl Into<DepositArgs> for CliDepositArgs {
    fn into(self) -> DepositArgs {
        DepositArgs {
            token: self.token,
            amount: self.amount,
            vault_id: self.vault_id,
        }
    }
}
