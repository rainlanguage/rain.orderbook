use std::str::FromStr;

use alloy::primitives::U256;
use alloy::sol_types::sol_data::Address;
use alloy::sol_types::SolCall;
use alloy::{hex::FromHex, primitives::Address};
use alloy_ethers_typecast::transaction::{
    ReadContractParameters, ReadableClientError, ReadableClientHttp,
};
use alloy_ethers_typecast::{
    multicall::{
        IMulticall3::{aggregate3Call, Call3},
        MULTICALL3_ADDRESS,
    },
    transaction::ReadContractParametersBuilderError,
};
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use rain_metaboard_subgraph::schema::__fields::MetaBoard::address;
use rain_orderbook_bindings::IERC20::{balanceOfCall, decimalsCall, nameCall, symbolCall};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TokenInfo {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TokenInfo);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ERC20 {
    pub rpc_url: Url,
    pub address: Address,
}

impl ERC20 {
    pub fn new(rpc_url: Url, address: Address) -> Self {
        Self { rpc_url, address }
    }

    async fn get_client(&self) -> Result<ReadableClientHttp, Error> {
        ReadableClientHttp::new_from_url(self.rpc_url.to_string()).map_err(|err| {
            Error::ReadableClientError {
                msg: format!("rpc url: {}", self.rpc_url),
                source: err,
            }
        })
    }

    pub async fn decimals(&self) -> Result<u8, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: decimalsCall {},
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?
            ._0)
    }

    pub async fn name(&self) -> Result<String, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: nameCall {},
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?
            ._0)
    }

    pub async fn symbol(&self) -> Result<String, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: symbolCall {},
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?
            ._0)
    }

    pub async fn balance_of(&self, account: Address) -> Result<U256, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: balanceOfCall { account: account },
            block_number: None,
            gas: None,
        };
        Ok(client
            .read(parameters)
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("account balance: {}", self.address),
                source: err,
            })?
            ._0)
    }

    pub async fn token_info(&self, multicall_address: Option<String>) -> Result<TokenInfo, Error> {
        let client = self.get_client().await?;

        let results = client
            .read(ReadContractParameters {
                gas: None,
                address: multicall_address
                    .map_or(Address::from_hex(MULTICALL3_ADDRESS).unwrap(), |s| {
                        Address::from_str(&s).unwrap_or(Address::default())
                    }),
                call: aggregate3Call {
                    calls: vec![
                        Call3 {
                            target: self.address,
                            allowFailure: false,
                            callData: decimalsCall {}.abi_encode().into(),
                        },
                        Call3 {
                            target: self.address,
                            allowFailure: false,
                            callData: nameCall {}.abi_encode().into(),
                        },
                        Call3 {
                            target: self.address,
                            allowFailure: false,
                            callData: symbolCall {}.abi_encode().into(),
                        },
                    ],
                },
                block_number: None,
            })
            .await
            .map_err(|err| Error::ReadableClientError {
                msg: format!("address: {}", self.address),
                source: err,
            })?;

        Ok(TokenInfo {
            decimals: decimalsCall::abi_decode_returns(&results.returnData[0].returnData, true)
                .map_err(|err| Error::SolTypesError {
                    msg: format!("address: {}", self.address),
                    source: err,
                })?
                ._0,
            name: nameCall::abi_decode_returns(&results.returnData[1].returnData, true)
                .map_err(|err| Error::SolTypesError {
                    msg: format!("address: {}", self.address),
                    source: err,
                })?
                ._0,
            symbol: symbolCall::abi_decode_returns(&results.returnData[2].returnData, true)
                .map_err(|err| Error::SolTypesError {
                    msg: format!("address: {}", self.address),
                    source: err,
                })?
                ._0,
        })
    }
}

const ERROR_MESSAGE: &str = "Failed to get token information: ";

#[derive(Debug, Error)]
pub enum Error {
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    ReadContractError {
        msg: String,
        #[source]
        source: ReadContractParametersBuilderError,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    ReadableClientError {
        msg: String,
        #[source]
        source: ReadableClientError,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    AbiDecodedErrorType {
        msg: String,
        #[source]
        source: AbiDecodedErrorType,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    AbiDecodeError {
        msg: String,
        #[source]
        source: AbiDecodeFailedErrors,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    SolTypesError {
        msg: String,
        #[source]
        source: alloy::sol_types::Error,
    },
}
