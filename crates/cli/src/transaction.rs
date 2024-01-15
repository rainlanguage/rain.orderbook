use alloy_primitives::U256;
use clap::Args;
use rain_orderbook_common::transaction::TransactionArgs;

#[derive(Args, Clone)]
pub struct CliTransactionArgs {
    #[arg(short, long, help = "Orderbook contract address")]
    pub orderbook_address: String,

    #[arg(short, long, help = "Derivation path of the Ledger wallet")]
    pub derivation_path: Option<usize>,

    #[arg(short, long, help = "Chain ID of the network")]
    pub chain_id: u64,

    #[arg(short, long, help = "RPC URL")]
    pub rpc_url: String,

    #[arg(short, long, help = "Max priority fee per gas (in wei)")]
    pub max_priority_fee_per_gas: Option<u128>,

    #[arg(short, long, help = "Max fee per gas (in wei)")]
    pub max_fee_per_gas: Option<u128>,
}

impl From<CliTransactionArgs> for TransactionArgs {
    fn from(val: CliTransactionArgs) -> Self {
        TransactionArgs {
            orderbook_address: val.orderbook_address,
            derivation_path: val.derivation_path,
            chain_id: val.chain_id,
            rpc_url: val.rpc_url,
            max_priority_fee_per_gas: val.max_priority_fee_per_gas.map(U256::from),
            max_fee_per_gas: val.max_fee_per_gas.map(U256::from),
        }
    }
}
