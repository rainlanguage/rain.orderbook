use alloy_primitives::{Address, U256};
use clap::Args;
use rain_orderbook_common::transaction::TransactionArgs;

#[derive(Args, Clone)]
pub struct CliTransactionArgs {
    #[arg(short, long, help = "Orderbook contract address")]
    pub orderbook_address: Address,

    #[arg(
        short,
        long,
        help = "Derivation index of the Ledger wallet address to use",
        default_value = "0"
    )]
    pub derivation_index: Option<usize>,

    #[arg(short, long, help = "Chain ID of the network")]
    pub chain_id: Option<u64>,

    #[arg(short, long, help = "RPC URL")]
    pub rpc_url: String,

    #[arg(short = 'p', long, help = "Max priority fee per gas (in wei)")]
    pub max_priority_fee_per_gas: Option<U256>,

    #[arg(short, long, help = "Max fee per gas (in wei)")]
    pub max_fee_per_gas: Option<U256>,
}

impl From<CliTransactionArgs> for TransactionArgs {
    fn from(val: CliTransactionArgs) -> Self {
        TransactionArgs {
            orderbook_address: val.orderbook_address,
            derivation_index: val.derivation_index,
            chain_id: val.chain_id,
            rpc_url: val.rpc_url,
            max_priority_fee_per_gas: val.max_priority_fee_per_gas,
            max_fee_per_gas: val.max_fee_per_gas,
        }
    }
}
