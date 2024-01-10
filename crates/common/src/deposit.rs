use alloy_primitives::{Address, U256};
use anyhow::Result;
use rain_orderbook_bindings::IOrderBookV3::depositCall;

pub struct DepositArgs {
    pub token: String,
    pub amount: u64,
    pub vault_id: u64,
}

impl DepositArgs {
    pub fn to_deposit_call(&self) -> Result<depositCall> {
        let token_address = self.token.parse::<Address>()?;
        let amount = U256::from(self.amount);
        let vault_id = U256::from(self.vault_id);

        Ok(depositCall {
            token: token_address,
            amount,
            vaultId: vault_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_deposit_call_valid() {
        let args = DepositArgs {
            token: "0xdcdee0E7a58Bba7e305dB3Abc42F4887CE8EF729".to_string(),
            amount: 100,
            vault_id: 1,
        };
        let result = args.to_deposit_call();
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_deposit_call_invalid_token() {
        let args = DepositArgs {
            token: "invalid".to_string(),
            amount: 100,
            vault_id: 1,
        };
        assert!(args.to_deposit_call().is_err());
    }
}
