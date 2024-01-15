use crate::transaction::CliTransactionArgs;
use alloy_ethers_typecast::client::LedgerClient;
use alloy_ethers_typecast::transaction::ExecutableTransaction;
use alloy_sol_types::SolCall;
use anyhow::Result;
use clap::Args;
use clap::FromArgMatches;
use clap::Parser;
use rain_orderbook_common::transaction::TransactionArgs;
use tracing::{debug, info};

#[derive(Parser, Clone)]
pub struct CliTransactionCallArgs<T: FromArgMatches + Args> {
    #[clap(flatten)]
    pub call_args: T,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

pub struct ExecuteTransaction {
    pub transaction_args: TransactionArgs,
}

impl<T: FromArgMatches + Args> From<CliTransactionCallArgs<T>> for ExecuteTransaction {
    fn from(value: CliTransactionCallArgs<T>) -> Self {
        Self {
            transaction_args: value.transaction_args.into(),
        }
    }
}

impl ExecuteTransaction {
    pub async fn send(&self, ledger_client: LedgerClient, call: impl SolCall) -> Result<()> {
        let request = self
            .transaction_args
            .to_transaction_request_with_call(call)
            .await?;

        let tx =
            ExecutableTransaction::from_alloy_transaction_request(request, ledger_client.client)
                .await?;

        info!("Awaiting signature from Ledger device");
        tx.execute().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    pub async fn connect_ledger(&mut self) -> Result<LedgerClient> {
        debug!("Connecting to Ledger device");
        self.transaction_args.clone().to_ledger_client().await
    }
}

pub trait Execute {
    async fn execute(&self) -> Result<()>;
}
