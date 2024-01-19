use alloy_ethers_typecast::client::LedgerClient;
use alloy_ethers_typecast::request_shim::AlloyTransactionRequest;
use alloy_ethers_typecast::transaction::ExecutableTransaction;
use alloy_primitives::{Address, U256, U64};
use alloy_sol_types::SolCall;
use anyhow::Result;
use clap::{Args, FromArgMatches, Parser};
use tracing::{debug, info};

#[derive(Args, Clone)]
pub struct TransactionArgs {
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

impl TransactionArgs {
    pub async fn try_into_transaction_request_with_call<T: SolCall>(
        &self,
        call: T,
    ) -> anyhow::Result<AlloyTransactionRequest> {
        let tx = AlloyTransactionRequest::default()
            .with_to(Some(self.orderbook_address.parse::<Address>()?))
            .with_data(Some(call.abi_encode().clone()))
            .with_chain_id(Some(U64::from(self.chain_id)))
            .with_max_priority_fee_per_gas(self.max_priority_fee_per_gas.map(U256::from))
            .with_max_fee_per_gas(self.max_fee_per_gas.map(U256::from));

        Ok(tx)
    }

    pub async fn to_ledger_client(self) -> anyhow::Result<LedgerClient> {
        LedgerClient::new(self.derivation_index, self.chain_id, self.rpc_url.clone()).await
    }
}

#[derive(Parser, Clone)]
pub struct CliTransactionCommandArgs<T: FromArgMatches + Args> {
    #[clap(flatten)]
    pub cmd_args: T,

    #[clap(flatten)]
    pub transaction_args: TransactionArgs,
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
    pub async fn send(&self, ledger_client: LedgerClient, call: impl SolCall) -> Result<()> {
        let request = self
            .transaction_args
            .try_into_transaction_request_with_call(call)
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
