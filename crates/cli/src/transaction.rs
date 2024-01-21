use alloy_ethers_typecast::client::LedgerClient;
use alloy_ethers_typecast::transaction::WritableClient;
use alloy_primitives::U256;
use alloy_sol_types::SolCall;
use anyhow::anyhow;
use anyhow::Result;
use clap::Args;
use clap::FromArgMatches;
use clap::Parser;
use rain_orderbook_common::transaction::TransactionArgs;
use tracing::{debug, info};

#[derive(Args, Clone)]
pub struct CliTransactionArgs {
    #[arg(short, long, help = "Orderbook contract address")]
    pub orderbook_address: String,

    #[arg(
        short,
        long,
        help = "Derivation index of the Ledger wallet address to use",
        default_value = "0"
    )]
    pub derivation_index: Option<usize>,

    #[arg(short, long, help = "Chain ID of the network")]
    pub chain_id: u64,

    #[arg(short, long, help = "RPC URL")]
    pub rpc_url: String,

    #[arg(short = 'p', long, help = "Max priority fee per gas (in wei)")]
    pub max_priority_fee_per_gas: Option<u128>,

    #[arg(short, long, help = "Max fee per gas (in wei)")]
    pub max_fee_per_gas: Option<u128>,
}

impl From<CliTransactionArgs> for TransactionArgs {
    fn from(val: CliTransactionArgs) -> Self {
        TransactionArgs {
            orderbook_address: val.orderbook_address,
            derivation_index: val.derivation_index,
            chain_id: val.chain_id,
            rpc_url: val.rpc_url,
            max_priority_fee_per_gas: val.max_priority_fee_per_gas.map(U256::from),
            max_fee_per_gas: val.max_fee_per_gas.map(U256::from),
        }
    }
}

#[derive(Parser, Clone)]
pub struct CliTransactionCommandArgs<T: FromArgMatches + Args> {
    #[clap(flatten)]
    pub cmd_args: T,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

pub struct ExecuteTransaction {
    pub transaction_args: TransactionArgs,
}

impl<T: FromArgMatches + Args> From<CliTransactionCommandArgs<T>> for ExecuteTransaction {
    fn from(value: CliTransactionCommandArgs<T>) -> Self {
        Self {
            transaction_args: value.transaction_args.into(),
        }
    }
}

impl ExecuteTransaction {
    pub async fn send(
        &self,
        ledger_client: LedgerClient,
        call: impl SolCall + Clone,
    ) -> Result<()> {
        let params = self
            .transaction_args
            .to_write_contract_parameters(call)
            .await?;

        let writable_client = WritableClient::new(ledger_client.client);
        writable_client
            .write(params)
            .await
            .map_err(|e| anyhow!(e))?;
        info!("Awaiting signature from Ledger device");

        Ok(())
    }

    pub async fn connect_ledger(&mut self) -> Result<LedgerClient> {
        debug!("Connecting to Ledger device");
        self.transaction_args.clone().to_ledger_client().await
    }
}
