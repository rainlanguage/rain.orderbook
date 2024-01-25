use crate::{error::WritableTransactionExecuteError, transaction::TransactionArgs};
use alloy_ethers_typecast::transaction::{WriteTransaction, WriteTransactionStatus};
use alloy_primitives::{hex::FromHexError, Address, U256};
use rain_orderbook_bindings::IOrderBookV3::withdrawCall;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct WithdrawArgs {
    pub token: String,
    pub vault_id: U256,
    pub target_amount: U256,
}

impl TryInto<withdrawCall> for WithdrawArgs {
    type Error = FromHexError;

    fn try_into(self) -> Result<withdrawCall, FromHexError> {
        Ok(withdrawCall {
            token: self.token.parse::<Address>()?,
            vaultId: self.vault_id,
            targetAmount: self.target_amount,
        })
    }
}

impl WithdrawArgs {
    /// Execute OrderbookV3 withdraw call
    pub async fn execute<S: Fn(WriteTransactionStatus<withdrawCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let ledger_client = transaction_args
            .clone()
            .try_into_ledger_client()
            .await
            .map_err(WritableTransactionExecuteError::TransactionArgs)?;

        let withdraw_call: withdrawCall = self.clone().try_into().map_err(|_| {
            WritableTransactionExecuteError::InvalidArgs(
                "Failed to parse address String into Address".into(),
            )
        })?;
        let params = transaction_args
            .try_into_write_contract_parameters(withdraw_call)
            .await
            .map_err(WritableTransactionExecuteError::TransactionArgs)?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await
            .map_err(WritableTransactionExecuteError::WritableClient)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_withdraw_args_try_into() {
        let args = WithdrawArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: U256::from(42),
            target_amount: U256::from(100),
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
