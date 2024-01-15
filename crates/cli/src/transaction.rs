use std::str::FromStr;

use anyhow::anyhow;
use clap::Args;
use ethers::middleware::gas_oracle::GasCategory;
use rain_orderbook_common::transaction::TransactionArgs;

#[derive(Clone)]
pub enum CliGasPriority {
    Slow,
    Average,
    Fast,
    Fastest,
}

impl FromStr for CliGasPriority {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "slow" => Ok(CliGasPriority::Slow),
            "average" => Ok(CliGasPriority::Average),
            "fast" => Ok(CliGasPriority::Fast),
            "fastest" => Ok(CliGasPriority::Fastest),
            _ => Err(anyhow!("Invalid gas priority")),
        }
    }
}

impl Into<GasCategory> for CliGasPriority {
    fn into(self) -> GasCategory {
        match self {
            CliGasPriority::Slow => GasCategory::SafeLow,
            CliGasPriority::Average => GasCategory::Standard,
            CliGasPriority::Fast => GasCategory::Fast,
            CliGasPriority::Fastest => GasCategory::Fastest,
        }
    }
}

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

    #[arg(
        short,
        long,
        help = "Gas Priority",
        value_enum,
        default_value = "average"
    )]
    pub gas_priority: CliGasPriority,
}

impl From<CliTransactionArgs> for TransactionArgs {
    fn from(val: CliTransactionArgs) -> Self {
        TransactionArgs {
            orderbook_address: val.orderbook_address,
            derivation_path: val.derivation_path,
            chain_id: val.chain_id,
            rpc_url: val.rpc_url,
            gas_priority: val.gas_priority.into(),
        }
    }
}
