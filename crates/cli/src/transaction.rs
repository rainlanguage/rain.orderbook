use clap::Args;

#[derive(Args)]
pub struct TransactionArgs {
    #[arg(short, long, help = "Orderbook contract address")]
    pub orderbook_address: String,

    #[arg(short, long, help = "Derivation path of the Ledger wallet")]
    pub derivation_path: Option<usize>,

    #[arg(short, long, help = "Chain ID of the network")]
    pub chain_id: u64,

    #[arg(short, long, help = "RPC URL")]
    pub rpc_url: String,

    #[arg(short, long, help = "Blocknative API Key")]
    pub blocknative_api_key: Option<String>,
}