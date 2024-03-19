use crate::transaction::{TransactionArgs, TransactionArgsError, WritableTransactionExecuteError};
use alloy_ethers_typecast::{
    ethers_address_to_alloy,
    transaction::{
        ReadContractParametersBuilder, ReadContractParametersBuilderError, ReadableClient,
        ReadableClientError, WritableClientError, WriteTransaction, WriteTransactionStatus,
    },
};
use alloy_primitives::{Address, U256};
use rain_orderbook_bindings::{
    IOrderBookV3::depositCall,
    IERC20::{allowanceCall, approveCall},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositError {
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),

    #[error(transparent)]
    ReadContractParametersBuilderError(#[from] ReadContractParametersBuilderError),

    #[error(transparent)]
    WritableClientError(#[from] WritableClientError),

    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),

    #[error(transparent)]
    TransactionArgsError(#[from] TransactionArgsError),
}

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
    /// Execute read IERC20 allowance call
    pub async fn read_allowance(
        &self,
        owner: Address,
        transaction_args: TransactionArgs,
    ) -> Result<U256, DepositError> {
        let readable_client = ReadableClient::new_from_url(transaction_args.rpc_url.clone())?;
        let parameters = ReadContractParametersBuilder::<allowanceCall>::default()
            .address(self.token)
            .call(allowanceCall {
                owner,
                spender: transaction_args.orderbook_address,
            })
            .build()?;
        let res = readable_client.read(parameters).await?;

        Ok(res._0)
    }

    /// Execute IERC20 approve call
    pub async fn execute_approve<S: Fn(WriteTransactionStatus<approveCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), DepositError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        // Check allowance already granted for this token and contract
        let current_allowance = self
            .read_allowance(
                ethers_address_to_alloy(ledger_client.client.address()),
                transaction_args.clone(),
            )
            .await?;

        // If more allowance is required, then call approve for the difference
        if current_allowance < self.amount {
            let approve_call = approveCall {
                spender: transaction_args.orderbook_address,
                amount: self.amount - current_allowance,
            };
            let params = transaction_args
                .try_into_write_contract_parameters(approve_call, self.token)
                .await?;

            WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
                .execute()
                .await?;
        }

        Ok(())
    }

    /// Execute OrderbookV3 deposit call
    pub async fn execute_deposit<S: Fn(WriteTransactionStatus<depositCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), DepositError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let deposit_call: depositCall = self.clone().into();
        let params = transaction_args
            .try_into_write_contract_parameters(deposit_call, transaction_args.orderbook_address)
            .await?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;

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
}
