use crate::types::order_detail;
use crate::utils::base10_str_to_u256;
use alloy_primitives::U256;
use alloy_primitives::{hex::FromHexError, ruint::ParseError, Address};
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
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
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

    fn try_into(self) -> Result<OrderV3, Self::Error> {
        let parsed: serde_json::Value = serde_json::from_str(&self.order_json)?;
        Ok(OrderV3 {
            owner: parsed["owner"]
                .as_str()
                .unwrap_or_default()
                .parse::<Address>()?,
            evaluable: EvaluableV3 {
                interpreter: parsed["evaluable"]["interpreter"]
                    .as_str()
                    .unwrap_or_default()
                    .parse::<Address>()?,
                store: parsed["evaluable"]["store"]
                    .as_str()
                    .unwrap_or_default()
                    .parse::<Address>()?,
                bytecode: vec![],
            },
            validInputs: parsed["validInputs"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| {
                    Ok(IO {
                        token: v["token"].as_str().unwrap_or_default().parse::<Address>()?,
                        decimals: v["decimals"].as_str().unwrap_or_default().parse::<u8>()?,
                        vaultId: v["vaultId"].as_str().unwrap_or_default().parse::<U256>()?,
                    })
                })
                .collect::<Result<Vec<IO>, OrderDetailError>>()?,
            validOutputs: parsed["validOutputs"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| {
                    Ok(IO {
                        token: v["token"].as_str().unwrap_or_default().parse::<Address>()?,
                        decimals: v["decimals"].as_str().unwrap_or_default().parse::<u8>()?,
                        vaultId: v["vaultId"].as_str().unwrap_or_default().parse::<U256>()?,
                    })
                })
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
            // This is a dummy order detail object
            // All of these values are actually ignored by the conversion
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
            // This is the JSON string that will actually be used for the conversion
            order_json: "{\"owner\":\"0x77199602114bdecb272ac9d5038d7e01cccec362\",\"handleIo\":true,\"evaluable\":{\"interpreter\":\"0xa10ac76b0681d3e8ca826b0a10299b99b2eb2452\",\"store\":\"0x948533c15d2d9eba8cec4deb0b72662cf57d4db1\",\"expression\":\"0x3c7e0efd0cf9bd539221eb15d9da4b9d97f8837b\"},\"validInputs\":[{\"token\":\"0x96b41289d90444b8add57e6f265db5ae8651df29\",\"decimals\":\"6\",\"vaultId\":\"0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5\"},{\"token\":\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\",\"decimals\":\"18\",\"vaultId\":\"0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5\"}],\"validOutputs\":[{\"token\":\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\",\"decimals\":\"18\",\"vaultId\":\"0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5\"},{\"token\":\"0x96b41289d90444b8add57e6f265db5ae8651df29\",\"decimals\":\"6\",\"vaultId\":\"0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5\"}]}".into()
        };

        let order_v2: OrderV3 = order_detail.try_into().unwrap();

        assert_eq!(
            order_v2.owner,
            "0x77199602114bdecb272ac9d5038d7e01cccec362"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            order_v2.evaluable.interpreter,
            "0xa10ac76b0681d3e8ca826b0a10299b99b2eb2452"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            order_v2.evaluable.store,
            "0x948533c15d2d9eba8cec4deb0b72662cf57d4db1"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            order_v2.validInputs[0].token,
            "0x96b41289d90444b8add57e6f265db5ae8651df29"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(order_v2.validInputs[0].decimals, 6);
        assert_eq!(
            order_v2.validInputs[0].vaultId,
            "0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5"
                .parse::<U256>()
                .unwrap()
        );
        assert_eq!(
            order_v2.validInputs[1].token,
            "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(order_v2.validInputs[1].decimals, 18);
        assert_eq!(
            order_v2.validInputs[1].vaultId,
            "0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5"
                .parse::<U256>()
                .unwrap()
        );
        assert_eq!(
            order_v2.validOutputs[0].token,
            "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(order_v2.validOutputs[0].decimals, 18);
        assert_eq!(
            order_v2.validOutputs[0].vaultId,
            "0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5"
                .parse::<U256>()
                .unwrap()
        );
        assert_eq!(
            order_v2.validOutputs[1].token,
            "0x96b41289d90444b8add57e6f265db5ae8651df29"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(order_v2.validOutputs[1].decimals, 6);
        assert_eq!(
            order_v2.validOutputs[1].vaultId,
            "0xb39eed084711b7e383c97ae5a9e0aa76e01f7a641457726ebfd912fe33dd67f5"
                .parse::<U256>()
                .unwrap()
        );
    }
}
