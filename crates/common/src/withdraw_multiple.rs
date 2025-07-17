#[cfg(not(target_family = "wasm"))]
use crate::transaction::TransactionArgs;
use crate::transaction::WritableTransactionExecuteError;
use crate::withdraw::WithdrawArgs;
use alloy::primitives::Bytes;
use alloy::sol_types::SolCall;
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::transaction::{WriteTransaction, WriteTransactionStatus};
use rain_orderbook_bindings::OrderBook::multicallCall;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct WithdrawMultipleArgs(pub Vec<WithdrawArgs>);

impl WithdrawMultipleArgs {
    /// Execute OrderbookV3 withdraw call
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<multicallCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        use crate::transaction::TRANSACTION_CONFIRMATIONS;

        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let withdraw_call = self.get_multicall().await?;
        let params = transaction_args.try_into_write_contract_parameters(
            withdraw_call,
            transaction_args.orderbook_address,
        )?;

        WriteTransaction::new(
            ledger_client.client,
            params,
            TRANSACTION_CONFIRMATIONS,
            transaction_status_changed,
        )
        .execute()
        .await?;

        Ok(())
    }

    pub async fn get_multicall(&self) -> Result<multicallCall, WritableTransactionExecuteError> {
        let mut withdraw_calldatas = Vec::new();
        for args in &self.0 {
            let withdraw_call: Bytes = args.get_withdraw_calldata().await?.into();
            withdraw_calldatas.push(withdraw_call);
        }

        let multicall = multicallCall {
            data: withdraw_calldatas,
        };
        Ok(multicall)
    }

    pub async fn get_calldata(&self) -> Result<Vec<u8>, WritableTransactionExecuteError> {
        let multicall = self.get_multicall().await?;
        let encoded = multicall.abi_encode();
        Ok(encoded)
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use alloy_ethers_typecast::gas_fee_middleware::GasFeeSpeed;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_calldata() {
        let withdraw_1 = WithdrawArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: U256::from(42),
            target_amount: U256::from(100),
        };
        let withdraw_2 = WithdrawArgs {
            token: Address::from_str("0xabcdef1234567890abcdef1234567890abcdef12").unwrap(),
            vault_id: U256::from(84),
            target_amount: U256::from(200),
        };

        let multi = WithdrawMultipleArgs(vec![withdraw_1.clone(), withdraw_2.clone()]);
        let calldata = multi.get_calldata().await.unwrap();

        let expected_calldata = multicallCall {
            data: vec![
                withdraw_1.get_withdraw_calldata().await.unwrap().into(),
                withdraw_2.get_withdraw_calldata().await.unwrap().into(),
            ],
        }
        .abi_encode();

        assert_eq!(calldata, expected_calldata);
    }

    #[tokio::test]
    async fn test_withdraw_call_try_into_write_contract_parameters() {
        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            orderbook_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(U256::from(200)),
            max_fee_per_gas: Some(U256::from(100)),
            gas_fee_speed: Some(GasFeeSpeed::Fast),
        };

        let withdraw_1 = WithdrawArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: U256::from(42),
            target_amount: U256::from(100),
        };
        let withdraw_2 = WithdrawArgs {
            token: Address::from_str("0xabcdef1234567890abcdef1234567890abcdef12").unwrap(),
            vault_id: U256::from(84),
            target_amount: U256::from(200),
        };

        let multi = WithdrawMultipleArgs(vec![withdraw_1.clone(), withdraw_2.clone()]);
        let call = multi.get_multicall().await.unwrap();

        let params = args
            .try_into_write_contract_parameters(call.clone(), Address::ZERO)
            .unwrap();
        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, call);
        assert_eq!(params.max_priority_fee_per_gas, Some(U256::from(200)));
        assert_eq!(params.max_fee_per_gas, Some(U256::from(100)));
    }
}
