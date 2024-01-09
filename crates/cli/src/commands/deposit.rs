use crate::transaction::CliTransactionArgs;
use alloy_ethers_typecast::client::LedgerClient;
use alloy_ethers_typecast::request_shim::AlloyTransactionRequest;
use alloy_ethers_typecast::transaction::ExecutableTransaction;
use alloy_primitives::{Address, U256, U64};
use alloy_sol_types::SolCall;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use rain_orderbook_bindings::IOrderBookV3::depositCall;

#[derive(Parser)]
pub struct Deposit {
    #[clap(flatten)]
    deposit_args: DepositArgs,
    #[clap(flatten)]
    transaction_args: CliTransactionArgs,
}

impl Deposit {
    pub async fn execute(self) -> Result<()> {
        let call_data = self.deposit_args.to_deposit_call()?.abi_encode();

        let tx = AlloyTransactionRequest::default()
            .with_to(self.transaction_args.orderbook_address.parse::<Address>()?)
            .with_data(call_data.clone())
            .with_chain_id(U64::from(self.transaction_args.chain_id));

        let ledger_client = LedgerClient::new(
            self.transaction_args.derivation_path,
            self.transaction_args.chain_id,
            self.transaction_args.rpc_url.clone(),
        )
        .await?;

        let transaction =
            ExecutableTransaction::from_alloy_transaction_request(tx, ledger_client.client).await?;

        transaction
            .execute()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

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
