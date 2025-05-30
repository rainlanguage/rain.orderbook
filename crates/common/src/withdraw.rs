#[cfg(not(target_family = "wasm"))]
use crate::transaction::TransactionArgs;
use crate::transaction::WritableTransactionExecuteError;
use alloy::primitives::{Address, U256};
use alloy::sol_types::SolCall;
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::transaction::{WriteTransaction, WriteTransactionStatus};
use rain_orderbook_bindings::IOrderBookV4::withdraw2Call;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct WithdrawArgs {
    pub token: Address,
    pub vault_id: U256,
    pub target_amount: U256,
}

impl From<WithdrawArgs> for withdraw2Call {
    fn from(val: WithdrawArgs) -> Self {
        withdraw2Call {
            token: val.token,
            vaultId: val.vault_id,
            targetAmount: val.target_amount,
            tasks: vec![],
        }
    }
}

impl WithdrawArgs {
    /// Execute OrderbookV3 withdraw call
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<withdraw2Call>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let withdraw_call: withdraw2Call = self.clone().into();
        let params = transaction_args.try_into_write_contract_parameters(
            withdraw_call,
            transaction_args.orderbook_address,
        )?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_withdraw_calldata(&self) -> Result<Vec<u8>, WritableTransactionExecuteError> {
        let withdraw_call: withdraw2Call = self.clone().into();
        Ok(withdraw_call.abi_encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_ethers_typecast::gas_fee_middleware::GasFeeSpeed;
    use std::str::FromStr;

    #[test]
    fn test_withdraw_args_into() {
        let args = WithdrawArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap(),
            vault_id: U256::from(42),
            target_amount: U256::from(100),
        };

        let withdraw_call: withdraw2Call = args.into();
        assert_eq!(
            withdraw_call.token,
            "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(withdraw_call.vaultId, U256::from(42));
        assert_eq!(withdraw_call.targetAmount, U256::from(100));
    }

    #[tokio::test]
    async fn test_get_withdraw_calldata() {
        let args = WithdrawArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: U256::from(42),
            target_amount: U256::from(100),
        };
        let calldata = args.get_withdraw_calldata().await.unwrap();

        let expected_calldata = withdraw2Call {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vaultId: U256::from(42),
            targetAmount: U256::from(100),
            tasks: vec![],
        }
        .abi_encode();

        assert_eq!(calldata, expected_calldata);
        assert_eq!(calldata.len(), 164);
    }

    #[test]
    fn test_withdraw_call_try_into_write_contract_parameters() {
        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            orderbook_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(U256::from(200)),
            max_fee_per_gas: Some(U256::from(100)),
            gas_fee_speed: Some(GasFeeSpeed::Fast),
        };
        let withdraw_call = withdraw2Call {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vaultId: U256::from(123),
            targetAmount: U256::from(456),
            tasks: vec![],
        };
        let params = args
            .try_into_write_contract_parameters(withdraw_call.clone(), Address::ZERO)
            .unwrap();
        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, withdraw_call);
        assert_eq!(params.max_priority_fee_per_gas, Some(U256::from(200)));
        assert_eq!(params.max_fee_per_gas, Some(U256::from(100)));
    }
}
