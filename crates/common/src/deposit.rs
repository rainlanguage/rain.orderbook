use crate::error::WritableTransactionExecuteError;
use crate::transaction::TransactionArgs;
use alloy_ethers_typecast::transaction::{WriteTransaction, WriteTransactionStatus};
use alloy_primitives::{Address, U256};
use rain_orderbook_bindings::{IOrderBookV3::depositCall, IERC20::approveCall};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DepositArgs {
    pub token: Address,
    pub vault_id: U256,
    pub amount: U256,
}

impl From<DepositArgs> for depositCall {
    fn from(val: DepositArgs) -> Self {
        depositCall {
            token: val.token,
            vaultId: val.vault_id,
            amount: val.amount,
        }
    }
}

impl DepositArgs {
    /// Build ERC20 approve call
    pub fn into_approve_call(&self, orderbook_address: Address) -> approveCall {
        approveCall {
            spender: orderbook_address,
            amount: self.amount,
        }
    }

    /// Execute ERC20 approve call
    pub async fn execute_approve<S: Fn(WriteTransactionStatus<approveCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let ledger_client = transaction_args
            .clone()
            .try_into_ledger_client()
            .await
            .map_err(WritableTransactionExecuteError::TransactionArgs)?;

        let approve_call = self.into_approve_call(transaction_args.orderbook_address);
        let params = transaction_args
            .try_into_write_contract_parameters(approve_call, self.token)
            .await
            .map_err(WritableTransactionExecuteError::TransactionArgs)?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await
            .map_err(WritableTransactionExecuteError::WritableClient)?;

        Ok(())
    }

    /// Execute OrderbookV3 deposit call
    pub async fn execute_deposit<S: Fn(WriteTransactionStatus<depositCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let ledger_client = transaction_args
            .clone()
            .try_into_ledger_client()
            .await
            .map_err(WritableTransactionExecuteError::TransactionArgs)?;

        let deposit_call: depositCall = self.clone().into();
        let params = transaction_args
            .try_into_write_contract_parameters(deposit_call, transaction_args.orderbook_address)
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
    use alloy_primitives::{hex, Address};

    #[test]
    fn test_deposit_args_into() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap(),
            vault_id: U256::from(42),
            amount: U256::from(100),
        };

        let deposit_call: depositCall = args.into();

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
            token: "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap(),
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
