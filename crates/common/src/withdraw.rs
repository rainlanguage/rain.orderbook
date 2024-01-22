use alloy_primitives::{hex::FromHexError, Address, U256};
use rain_orderbook_bindings::IOrderBookV3::withdrawCall;
use std::convert::TryInto;

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
