use crate::transaction::{TransactionArgs, TransactionArgsError, WritableTransactionExecuteError};
use alloy::primitives::{Address, U256};
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::transaction::{
    ReadContractParametersBuilder, ReadContractParametersBuilderError, ReadableClient,
    ReadableClientError, WritableClientError,
};
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::{
    ethers_address_to_alloy,
    transaction::{WriteTransaction, WriteTransactionStatus},
};
use rain_orderbook_bindings::{
    IOrderBookV4::deposit2Call,
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

impl From<DepositArgs> for deposit2Call {
    fn from(val: DepositArgs) -> Self {
        deposit2Call {
            token: val.token,
            vaultId: val.vault_id,
            amount: val.amount,
            tasks: vec![],
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
    #[cfg(not(target_family = "wasm"))]
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

    pub async fn get_approve_calldata(
        &self,
        transaction_args: TransactionArgs,
        current_allowance: U256,
    ) -> Result<Vec<u8>, WritableTransactionExecuteError> {
        let approve_call = approveCall {
            spender: transaction_args.orderbook_address,
            amount: self.amount,
        };
        Ok(approve_call.abi_encode())
    }

    /// Execute OrderbookV3 deposit call
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute_deposit<S: Fn(WriteTransactionStatus<deposit2Call>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), DepositError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let deposit_call: deposit2Call = self.clone().into();
        let params = transaction_args
            .try_into_write_contract_parameters(deposit_call, transaction_args.orderbook_address)
            .await?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_deposit_calldata(&self) -> Result<Vec<u8>, WritableTransactionExecuteError> {
        let deposit_call: deposit2Call = self.clone().into();
        Ok(deposit_call.abi_encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn test_deposit_args_into() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap(),
            vault_id: U256::from(42),
            amount: U256::from(100),
        };

        let deposit_call: deposit2Call = args.into();

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
