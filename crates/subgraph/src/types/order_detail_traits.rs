use crate::types::order_detail;
use crate::utils::base10_str_to_u256;
use alloy::primitives::{
    hex::{decode, FromHexError},
    ruint::ParseError,
    Address,
};
use alloy::sol_types::SolValue;
use rain_orderbook_bindings::IOrderBookV4::{OrderV3, IO};
use std::num::TryFromIntError;
use thiserror::Error;

use super::order_detail::BigInt;

#[derive(Error, Debug)]
pub enum OrderDetailError {
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    AbiDecode(#[from] alloy::sol_types::Error),
}

impl TryInto<IO> for order_detail::Vault {
    type Error = OrderDetailError;

    fn try_into(self) -> Result<IO, OrderDetailError> {
        Ok(IO {
            token: self.token.address.0.parse::<Address>()?,
            decimals: self
                .token
                .decimals
                .unwrap_or(BigInt("0".into()))
                .0
                .parse::<u8>()?,

            // Vault ID returned from the subgraph is the base-10 value in a string *without* a "0x" prefix
            // See https://github.com/rainlanguage/rain.orderbook/issues/315
            vaultId: base10_str_to_u256(self.vault_id.0.as_str())?,
        })
    }
}

impl TryInto<OrderV3> for order_detail::Order {
    type Error = OrderDetailError;

    fn try_into(self) -> Result<OrderV3, Self::Error> {
        let order = rain_orderbook_bindings::IOrderBookV4::OrderV3::abi_decode(
            &decode(self.order_bytes.0)?,
            true,
        )?;
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use alloy::primitives::U256;

    use super::*;
    use crate::types::order_detail::{BigInt, Bytes};

    #[test]
    fn test_try_into_call() {
        let order_detail = order_detail::Order {
            // This is a dummy order detail object
            // All of these values are actually ignored by the conversion
            id: Bytes("1".into()),
            order_hash: Bytes("".into()),
            add_events: vec![],
            timestamp_added: BigInt("0".into()),
            owner: Bytes("".into()),
            active: true,
            inputs: vec![],
            outputs: vec![],
            meta: None,
            // Only the order_bytes field is used for the conversion
            order_bytes: Bytes("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000006171c21b2e553c59a64d1337211b77c367cefe5d00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000379b966dc6b117dd47b5fc5308534256a4ab1bcc0000000000000000000000006e4b01603edbda617002a077420e98c86595748e000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000950000000000000000000000000000000000000000000000000000000000000002ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000b1a2bc2ec5000000000000000000000000000000000000000000000000000000000000000000015020000000c020200020110000001100001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000050c5725949a6f0c72e6c4a641f24049a917db0cb000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda0291300000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001".into()),
        };

        let order_v3: OrderV3 = order_detail.try_into().unwrap();

        assert_eq!(
            order_v3.owner,
            "0x6171c21b2e553c59a64d1337211b77c367cefe5d"
                .parse::<Address>()
                .unwrap()
        );

        assert_eq!(
            order_v3.validInputs[0].token,
            "0x50c5725949a6f0c72e6c4a641f24049a917db0cb"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(order_v3.validInputs[0].decimals, 18);
        assert_eq!(order_v3.validInputs[0].vaultId, U256::from(1));

        assert_eq!(
            order_v3.validOutputs[0].token,
            "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"
                .parse::<Address>()
                .unwrap()
        );

        assert_eq!(order_v3.validOutputs[0].decimals, 6);
        assert_eq!(order_v3.validOutputs[0].vaultId, U256::from(1));
    }
}
