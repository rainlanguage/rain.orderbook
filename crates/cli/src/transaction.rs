use clap::Args;
use rain_orderbook_common::transaction::TransactionArgs;

#[derive(Args)]
pub struct CliTransactionArgs {
    #[arg(short, long, help = "Orderbook contract address")]
    pub orderbook_address: String,

    #[arg(short, long, help = "Derivation path of the Ledger wallet")]
    pub derivation_path: Option<usize>,

    #[arg(short, long, help = "Chain ID of the network")]
    pub chain_id: u64,

    #[arg(short, long, help = "RPC URL")]
    pub rpc_url: String,
}

impl Into<TransactionArgs> for CliTransactionArgs {
    fn into(self) -> TransactionArgs {
        TransactionArgs {
            orderbook_address: self.orderbook_address,
            derivation_path: self.derivation_path,
            chain_id: self.chain_id,
            rpc_url: self.rpc_url,
        }
    }
}
