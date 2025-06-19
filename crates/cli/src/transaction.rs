use alloy::primitives::Address;
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

    #[arg(
        short = 'p',
        long,
        help = "Max priority fee per gas (in wei)",
        conflicts_with("gas_fee_speed")
    )]
    pub max_priority_fee_per_gas: Option<u128>,

    #[arg(
        short,
        long,
        help = "Max fee per gas (in wei)",
        conflicts_with("gas_fee_speed")
    )]
    pub max_fee_per_gas: Option<u128>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_from_cli_transaction_args() {
        let cli_args = CliTransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: Some(1),
            chain_id: Some(2),
            rpc_url: "http://localhost:8545".to_string(),
            max_priority_fee_per_gas: Some(U256::from(100)),
            max_fee_per_gas: Some(U256::from(1000)),
        };

        let transaction_args: TransactionArgs = cli_args.into();

        assert_eq!(transaction_args.orderbook_address, Address::ZERO);
        assert_eq!(transaction_args.derivation_index, Some(1));
        assert_eq!(transaction_args.chain_id, Some(2));
        assert_eq!(transaction_args.rpc_url, "http://localhost:8545");
        assert_eq!(
            transaction_args.max_priority_fee_per_gas,
            Some(U256::from(100))
        );
        assert_eq!(transaction_args.max_fee_per_gas, Some(U256::from(1000)));
        assert_eq!(transaction_args.gas_fee_speed, Some(GasFeeSpeed::Fast));

        let orderbook_address = Address::random();
        let cli_args = CliTransactionArgs {
            orderbook_address,
            derivation_index: None,
            chain_id: None,
            rpc_url: "http://localhost:8545".to_string(),
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_fee_speed: None,
        };

        let transaction_args: TransactionArgs = cli_args.into();

        assert_eq!(transaction_args.orderbook_address, orderbook_address);
        assert_eq!(transaction_args.derivation_index, None);
        assert_eq!(transaction_args.chain_id, None);
        assert_eq!(transaction_args.rpc_url, "http://localhost:8545");
        assert_eq!(transaction_args.max_priority_fee_per_gas, None);
        assert_eq!(transaction_args.max_fee_per_gas, None);
        assert_eq!(transaction_args.gas_fee_speed, None);
    }

    #[test]
    fn test_from_cli_gas_fee_speed() {
        assert_eq!(GasFeeSpeed::from(CliGasFeeSpeed::Slow), GasFeeSpeed::Slow);
        assert_eq!(
            GasFeeSpeed::from(CliGasFeeSpeed::Medium),
            GasFeeSpeed::Medium
        );
        assert_eq!(GasFeeSpeed::from(CliGasFeeSpeed::Fast), GasFeeSpeed::Fast);
        assert_eq!(
            GasFeeSpeed::from(CliGasFeeSpeed::Fastest),
            GasFeeSpeed::Fastest
        );
    }
}
