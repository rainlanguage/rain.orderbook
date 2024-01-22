use crate::error::WritableTransactionExecuteError;
use crate::transaction::TransactionArgs;
use alloy_ethers_typecast::transaction::WriteTransaction;
use alloy_ethers_typecast::{ethers_address_to_alloy, transaction::WriteTransactionStatus};
use alloy_primitives::{hex::FromHexError, Address, U256};
use rain_orderbook_bindings::{IOrderBookV3::depositCall, IERC20::approveCall};
use std::convert::TryInto;

#[derive(Clone)]
pub struct DepositArgs {
    pub token: String,
    pub vault_id: U256,
    pub amount: U256,
}

impl TryInto<depositCall> for DepositArgs {
    type Error = FromHexError;

    fn try_into(self) -> Result<depositCall, FromHexError> {
        Ok(depositCall {
            token: self.token.parse()?,
            vaultId: self.vault_id,
            amount: self.amount,
        })
    }
}

impl DepositArgs {
    pub fn into_approve_call(self, spender: Address) -> approveCall {
        approveCall {
            spender,
            amount: self.amount,
        }
    }

    pub async fn execute<
        A: Fn(WriteTransactionStatus<approveCall>),
        D: Fn(WriteTransactionStatus<depositCall>),
        S: Fn(),
    >(
        &self,
        transaction_args: TransactionArgs,
        approve_transaction_status_changed: A,
        deposit_transaction_status_changed: D,
        approve_transaction_success: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        self.execute_approve(transaction_args.clone(), approve_transaction_status_changed)
            .await?;
        (approve_transaction_success)();
        self.execute_deposit(transaction_args, deposit_transaction_status_changed)
            .await?;

        Ok(())
    }

    /// Execute ERC20 approve call
    async fn execute_approve<S: Fn(WriteTransactionStatus<approveCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let ledger_client = transaction_args
            .clone()
            .try_into_ledger_client()
            .await
            .map_err(|e| WritableTransactionExecuteError::LedgerClient(e))?;

        let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
        let approve_call = self.clone().into_approve_call(ledger_address);
        let params = transaction_args
            .try_into_write_contract_parameters(approve_call)
            .await
            .map_err(|e| WritableTransactionExecuteError::TransactionArgs(e))?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await
            .map_err(|e| WritableTransactionExecuteError::WritableClient(e))?;

        Ok(())
    }

    /// Execute OrderbookV3 deposit call
    async fn execute_deposit<S: Fn(WriteTransactionStatus<depositCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let ledger_client = transaction_args
            .clone()
            .try_into_ledger_client()
            .await
            .map_err(|e| WritableTransactionExecuteError::LedgerClient(e))?;

        let deposit_call: depositCall = self.clone().try_into().map_err(|_| {
            WritableTransactionExecuteError::InvalidArgs(
                "Failed to parse address String into Address".into(),
            )
        })?;
        let params = transaction_args
            .try_into_write_contract_parameters(deposit_call)
            .await
            .map_err(|e| WritableTransactionExecuteError::TransactionArgs(e))?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await
            .map_err(|e| WritableTransactionExecuteError::WritableClient(e))?;

        Ok(())
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
            vault_id: U256::from(42),
            amount: U256::from(100),
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
    fn test_deposit_args_into_approve_call() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: U256::from(42),
            amount: U256::from(100),
        };
        let spender_address = Address::repeat_byte(0x11);
        let approve_call: approveCall = args.into_approve_call(spender_address);

        assert_eq!(approve_call.amount, U256::from(100));
        assert_eq!(
            approve_call.spender,
            hex!("1111111111111111111111111111111111111111")
        );
    }
}
