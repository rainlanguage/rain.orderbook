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
            post: vec![],
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
        let params = transaction_args
            .try_into_write_contract_parameters(withdraw_call, transaction_args.orderbook_address)
            .await?;

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
}
