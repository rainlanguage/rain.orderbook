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
        let readable_client = ReadableClient::new_from_urls(transaction_args.rpcs.clone())?;
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
            let params =
                transaction_args.try_into_write_contract_parameters(approve_call, self.token)?;

            WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
                .execute()
                .await?;
        }

        Ok(())
    }

    pub async fn get_approve_calldata(
        &self,
        transaction_args: TransactionArgs,
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
            .try_into_write_contract_parameters(deposit_call, transaction_args.orderbook_address)?;

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
    use alloy::primitives::{Address, B256};
    use alloy_ethers_typecast::{gas_fee_middleware::GasFeeSpeed, rpc::Response};
    use httpmock::MockServer;
    use std::str::FromStr;

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

    #[tokio::test]
    async fn test_read_allowance() {
        let rpc_server = MockServer::start_async().await;

        rpc_server.mock(|when, then| {
            when.path("/rpc").body_contains("0xdd62ed3e");
            then.body(
                Response::new_success(1, &B256::left_padding_from(&[200u8]).to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });

        let args = DepositArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: U256::from(42),
            amount: U256::from(100),
        };

        let res = args
            .read_allowance(
                Address::ZERO,
                TransactionArgs {
                    rpcs: vec![rpc_server.url("/rpc")],
                    orderbook_address: Address::ZERO,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(res, U256::from(200));
    }

    #[tokio::test]
    async fn test_get_deposit_calldata() {
        let args = DepositArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: U256::from(42),
            amount: U256::from(100),
        };
        let calldata = args.get_deposit_calldata().await.unwrap();

        let deposit_call = deposit2Call {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vaultId: U256::from(42),
            amount: U256::from(100),
            tasks: vec![],
        };
        let expected_calldata = deposit_call.abi_encode();

        assert_eq!(calldata, expected_calldata);
        assert_eq!(calldata.len(), 164);
    }

    #[test]
    fn test_deposit_call_try_into_write_contract_parameters() {
        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            orderbook_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(U256::from(200)),
            max_fee_per_gas: Some(U256::from(100)),
            gas_fee_speed: Some(GasFeeSpeed::Fast),
        };
        let deposit_call = deposit2Call {
            token: Address::ZERO,
            vaultId: U256::from(42),
            amount: U256::from(100),
            tasks: vec![],
        };
        let params = args
            .try_into_write_contract_parameters(deposit_call.clone(), Address::ZERO)
            .unwrap();
        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, deposit_call);
        assert_eq!(params.max_priority_fee_per_gas, Some(U256::from(200)));
        assert_eq!(params.max_fee_per_gas, Some(U256::from(100)));
    }

    #[tokio::test]
    async fn test_get_approve_calldata() {
        let args = DepositArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: U256::from(42),
            amount: U256::from(100),
        };
        let calldata = args
            .get_approve_calldata(TransactionArgs {
                rpcs: vec!["https://mainnet.infura.io/v3/".to_string()],
                orderbook_address: Address::ZERO,
                ..Default::default()
            })
            .await
            .unwrap();

        let approve_call = approveCall {
            spender: Address::ZERO,
            amount: U256::from(100),
        };
        let expected_calldata = approve_call.abi_encode();

        assert_eq!(calldata, expected_calldata);
        assert_eq!(calldata.len(), 68);
    }

    #[test]
    fn test_approve_call_try_into_write_contract_parameters() {
        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            orderbook_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(U256::from(200)),
            max_fee_per_gas: Some(U256::from(100)),
            gas_fee_speed: Some(GasFeeSpeed::Fast),
        };
        let approve_call = approveCall {
            spender: Address::ZERO,
            amount: U256::from(100),
        };
        let params = args
            .try_into_write_contract_parameters(approve_call.clone(), Address::ZERO)
            .unwrap();
        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, approve_call);
        assert_eq!(params.max_priority_fee_per_gas, Some(U256::from(200)));
        assert_eq!(params.max_fee_per_gas, Some(U256::from(100)));
    }
}
