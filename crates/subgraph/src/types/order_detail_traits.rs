use crate::types::order_detail;
use crate::utils::base10_str_to_u256;
use alloy_primitives::{hex::FromHexError, ruint::ParseError, Address, U256};
use rain_orderbook_bindings::IOrderBookV4::{EvaluableV3, OrderV3, IO};
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderDetailError {
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
}

impl TryInto<IO> for order_detail::Io {
    type Error = OrderDetailError;

    fn try_into(self) -> Result<IO, OrderDetailError> {
        Ok(IO {
            token: self.token_vault.token.id.into_inner().parse::<Address>()?,
            decimals: self.token_vault.token.decimals.try_into()?,

            // Vault ID returned from the subgraph is the base-10 value in a string *without* a "0x" prefix
            // See https://github.com/rainlanguage/rain.orderbook/issues/315
            vaultId: base10_str_to_u256(self.token_vault.vault_id.0.as_str())?,
        })
    }
}

impl TryInto<OrderV3> for order_detail::Order {
    type Error = OrderDetailError;

    fn try_into(self) -> Result<OrderV3, OrderDetailError> {
        Ok(OrderV3 {
            owner: self.owner.id.0.parse::<Address>()?,
            evaluable: EvaluableV3 {
                interpreter: self.interpreter.0.parse::<Address>()?,
                store: self.interpreter_store.0.parse::<Address>()?,
                bytecode: vec![],
            },
            validInputs: self
                .valid_inputs
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<IO>, OrderDetailError>>()?,
            validOutputs: self
                .valid_outputs
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<IO>, OrderDetailError>>()?,
            nonce: U256::from(0).into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::types::order_detail::{
        Account, BigInt, Bytes, Erc20, Io, RainMetaV1, TokenVault, Vault,
    };

    #[test]
    fn test_try_into_call() {
        let order_detail = order_detail::Order {
            id: "1".into(),
            owner: Account {
                id: Bytes("0x0000000000000000000000000000000000000001".into()),
            },
            order_active: true,
            interpreter: Bytes("0x0000000000000000000000000000000000000002".into()),
            interpreter_store: Bytes("0x0000000000000000000000000000000000000003".into()),
            expression_deployer: Bytes("".into()),
            expression: Bytes("0x0000000000000000000000000000000000000004".into()),
            timestamp: BigInt("".into()),
            handle_io: true,
            valid_inputs: Some(vec![Io {
                token_vault: TokenVault {
                    id: "".into(),
                    vault_id: BigInt("1".into()),
                    vault: Vault {
                        owner: Account {
                            id: Bytes("".into()),
                        },
                    },
                    token: Erc20 {
                        id: cynic::Id::new("0x0000000000000000000000000000000000000005"),
                        name: "".into(),
                        symbol: "ABC".into(),
                        decimals: 18,
                    },
                },
            }]),
            valid_outputs: Some(vec![Io {
                token_vault: TokenVault {
                    id: "".into(),
                    vault_id: BigInt("2".into()),
                    vault: Vault {
                        owner: Account {
                            id: Bytes("".into()),
                        },
                    },
                    token: Erc20 {
                        id: cynic::Id::new("0x0000000000000000000000000000000000000006"),
                        name: "".into(),
                        symbol: "DEF".into(),
                        decimals: 18,
                    },
                },
            }]),
            meta: Some(RainMetaV1 {
                meta_bytes: Bytes("0x1".into()),
                content: vec![],
            }),
        };

        let order_v2: OrderV3 = order_detail.try_into().unwrap();

        assert_eq!(
            order_v2.owner,
            "0x0000000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            order_v2.evaluable.interpreter,
            "0x0000000000000000000000000000000000000002"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            order_v2.evaluable.store,
            "0x0000000000000000000000000000000000000003"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(order_v2.evaluable.bytecode, vec![] as Vec<u8>);
        assert_eq!(
            order_v2.validInputs[0].token,
            "0x0000000000000000000000000000000000000005"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            order_v2.validOutputs[0].token,
            "0x0000000000000000000000000000000000000006"
                .parse::<Address>()
                .unwrap()
        );
    }
}
