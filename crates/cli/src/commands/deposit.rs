use crate::transaction::TransactionArgs;
use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use ethers_signers::{HDPath, Ledger};
use rain_orderbook_bindings::IOrderBookV3::depositCall;
use rain_orderbook_transactions::execute::execute_transaction;

#[derive(Parser)]
pub struct Deposit {
    #[clap(flatten)]
    deposit_args: DepositArgs,
    #[clap(flatten)]
    transaction_args: TransactionArgs,
}

impl Deposit {
    pub async fn execute(self) -> Result<()> {
        let deposit_call = self.deposit_args.to_deposit_call()?;
        let call_data = deposit_call.abi_encode();
        execute_transaction(
            call_data,
            self.transaction_args.orderbook_address.parse::<Address>()?,
            U256::from(0),
            self.transaction_args.rpc_url,
            Ledger::new(
                HDPath::LedgerLive(self.transaction_args.derivation_path.unwrap_or(0)),
                self.transaction_args.chain_id,
            )
            .await?,
            self.transaction_args.blocknative_api_key,
        )
        .await?;
        Ok(())
    }
}

#[derive(Args)]
pub struct DepositArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The amount to deposit")]
    amount: u64,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,
}

impl DepositArgs {
    pub fn to_deposit_call(&self) -> Result<depositCall> {
        let token_address = self.token.parse::<Address>()?;
        let amount = U256::from(self.amount);
        let vault_id = U256::from(self.vault_id);

        Ok(depositCall {
            token: token_address,
            amount,
            vaultId: vault_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_deposit_call_valid() {
        let args = DepositArgs {
            token: "0xdcdee0E7a58Bba7e305dB3Abc42F4887CE8EF729".to_string(),
            amount: 100,
            vault_id: 1,
        };
        let result = args.to_deposit_call();
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_deposit_call_invalid_token() {
        let args = DepositArgs {
            token: "invalid".to_string(),
            amount: 100,
            vault_id: 1,
        };
        assert!(args.to_deposit_call().is_err());
    }
}

