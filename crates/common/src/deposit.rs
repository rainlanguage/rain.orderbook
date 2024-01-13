use alloy_primitives::U256;
use anyhow::Result;
use rain_orderbook_bindings::IOrderBookV3::depositCall;
use std::convert::TryInto;

pub struct DepositArgs {
    pub token: String,
    pub vault_id: u64,
    pub amount: u64,
}

impl TryInto<depositCall> for DepositArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<depositCall> {
        Ok(depositCall {
            token: self.token.parse()?,
            vaultId: U256::from(self.vault_id),
            amount: U256::from(self.amount),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;

    #[test]
    fn test_deposit_args_try_into() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: 42,
            amount: 100,
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
}
