use crate::{
    execute::Execute,
    transaction::{CliTransactionCommandArgs, ExecuteTransaction},
};
use alloy_ethers_typecast::ethers_address_to_alloy;
use alloy_primitives::{Address, U256};
use anyhow::Result;
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::{IOrderBookV3::depositCall, IERC20::approveCall};
use std::convert::TryInto;
use tracing::info;

#[derive(Args, Clone)]
pub struct DepositArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,

    #[arg(short, long, help = "The amount to deposit")]
    amount: u64,
}

impl TryInto<depositCall> for DepositArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<depositCall> {
        Ok(depositCall {
            token: self.token.parse::<Address>()?,
            vaultId: U256::from(self.vault_id),
            amount: U256::from(self.amount),
        })
    }
}

impl DepositArgs {
    pub fn try_into_approve_call(self, spender: Address) -> Result<approveCall> {
        Ok(approveCall {
            spender,
            amount: U256::from(self.amount),
        })
    }
}

pub type Deposit = CliTransactionCommandArgs<DepositArgs>;

impl Execute for Deposit {
    async fn execute(&self) -> Result<()> {
        // Prepare deposit call
        let deposit_args: DepositArgs = self.cmd_args.clone().into();
        let deposit_call: depositCall = deposit_args.clone().try_into()?;

        // Prepare approve call
        let mut execute_tx: ExecuteTransaction = self.clone().into();
        let ledger_client = execute_tx.connect_ledger().await?;
        let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
        let approve_call: approveCall =
            deposit_args.clone().try_into_approve_call(ledger_address)?;

        info!("Step 1/2: Approve token transfer");
        execute_tx.send(ledger_client, approve_call).await?;

        info!("Step 2/2: Deposit tokens into vault");
        let ledger_client = execute_tx.connect_ledger().await?;
        execute_tx.send(ledger_client, deposit_call).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex, Address};

    #[test]
    fn test_deposit_args_try_into() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: 42,
            amount: 100,
        };

        let result: Result<depositCall, _> = args.try_into();

        match result {
            Ok(_) => (),
            Err(e) => panic!("Unexpected error: {}", e),
        }

        assert!(result.is_ok());

        let deposit_call = result.unwrap();
        assert_eq!(
            deposit_call.token,
            "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(deposit_call.vaultId, U256::from(42));
        assert_eq!(deposit_call.amount, U256::from(100));
    }

    #[test]
    fn test_deposit_args_try_into_approve_call() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: 42,
            amount: 100,
        };
        let spender_address = Address::repeat_byte(0x11);
        let result: Result<approveCall, _> = args.try_into_approve_call(spender_address);

        match result {
            Ok(_) => (),
            Err(e) => panic!("Unexpected error: {}", e),
        }

        assert!(result.is_ok());

        let approve_call = result.unwrap();
        assert_eq!(approve_call.amount, U256::from(100));
        assert_eq!(
            approve_call.spender,
            hex!("1111111111111111111111111111111111111111")
        );
    }
}
