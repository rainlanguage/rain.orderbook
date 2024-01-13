use crate::transaction::CliTransactionArgs;
use alloy_ethers_typecast::transaction::ExecutableTransaction;
use alloy_sol_types::SolCall;
use anyhow::Result;
use clap::Args;
use clap::FromArgMatches;
use clap::Parser;
use rain_orderbook_common::transaction::TransactionArgs;
use tracing::info;

#[derive(Parser)]
pub struct ExecutableTransactionCall<T: FromArgMatches + Args> {
    #[clap(flatten)]
    pub call_args: T,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

impl<T: FromArgMatches + Args> ExecutableTransactionCall<T> {
    pub async fn execute_transaction_call(self, call: impl SolCall) -> Result<()> {
        let tx_args: TransactionArgs = self.transaction_args.into();
        let request = tx_args.to_transaction_request_with_call(call).await?;

        info!("Connecting to Ledger device");
        let ledger_client = tx_args.to_ledger_client().await?;

        let tx =
            ExecutableTransaction::from_alloy_transaction_request(request, ledger_client.client)
                .await?;

        info!("Awaiting signature from Ledger device");
        tx.execute().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}

pub trait Execute {
    async fn execute(self) -> Result<()>;
}
