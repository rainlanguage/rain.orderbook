use alloy_primitives::{Address, U256};
use anyhow::Result;
use rain_orderbook_bindings::IOrderBookV3::withdrawCall;
use std::convert::TryInto;

pub struct WithdrawArgs {
    pub token: String,
    pub vault_id: u64,
    pub target_amount: u64,
}

impl TryInto<withdrawCall> for WithdrawArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<withdrawCall> {
        let token = self.token.parse::<Address>()?;
        let vault_id = U256::from(self.vault_id);
        let target_amount = U256::from(self.target_amount);

        Ok(withdrawCall {
            token: token,
            vaultId: vault_id,
            targetAmount: target_amount,
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
