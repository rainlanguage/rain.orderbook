use crate::{
    execute::Execute,
    transaction::{CliTransactionCommandArgs, ExecuteTransaction},
};
use alloy_primitives::{Address, U256};
use anyhow::Result;
use clap::Args;
use rain_orderbook_bindings::IOrderBookV3::withdrawCall;

pub type Withdraw = CliTransactionCommandArgs<WithdrawArgs>;

impl Execute for Withdraw {
    async fn execute(&self) -> Result<()> {
        let mut execute_tx: ExecuteTransaction = self.clone().into();
        let withdraw_call: withdrawCall = self.cmd_args.clone().try_into()?;

        let ledger_client = execute_tx.connect_ledger().await?;
        execute_tx.send(ledger_client, withdraw_call).await
    }
}

#[derive(Args, Clone)]
pub struct WithdrawArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,

    #[arg(short = 'a', long, help = "The target amount to withdraw")]
    target_amount: u64,
}

impl TryInto<withdrawCall> for WithdrawArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<withdrawCall> {
        Ok(withdrawCall {
            token: self.token.parse::<Address>()?,
            vaultId: U256::from(self.vault_id),
            targetAmount: U256::from(self.target_amount),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_withdraw_args_try_into() {
        let args = WithdrawArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: 42,
            target_amount: 100,
        };

        let result: Result<withdrawCall, _> = args.try_into();

        match result {
            Ok(_) => (),
            Err(e) => panic!("Unexpected error: {}", e),
        }

        assert!(result.is_ok());

        let withdraw_call = result.unwrap();
        assert_eq!(
            withdraw_call.token,
            "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(withdraw_call.vaultId, U256::from(42));
        assert_eq!(withdraw_call.targetAmount, U256::from(100));
    }
}
