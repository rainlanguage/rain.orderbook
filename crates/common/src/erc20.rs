use std::str::FromStr;

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
use rain_orderbook_bindings::IERC20::{decimalsCall, nameCall, symbolCall};
#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TokenInfo {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(TokenInfo);

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
        Ok(ReadableClientHttp::new_from_url(self.rpc_url.to_string())?)
    }

    pub async fn decimals(&self) -> Result<u8, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: decimalsCall {},
            block_number: None,
        };
        Ok(client.read(parameters).await?._0)
    }

    pub async fn name(&self) -> Result<String, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: nameCall {},
            block_number: None,
        };
        Ok(client.read(parameters).await?._0)
    }

    pub async fn symbol(&self) -> Result<String, Error> {
        let client = self.get_client().await?;
        let parameters = ReadContractParameters {
            address: self.address,
            call: symbolCall {},
            block_number: None,
        };
        Ok(client.read(parameters).await?._0)
    }

    pub async fn token_info(&self, multicall_address: Option<String>) -> Result<TokenInfo, Error> {
        let client = self.get_client().await?;

        let results = client
            .read(ReadContractParameters {
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
            .await?;

        Ok(TokenInfo {
            decimals: decimalsCall::abi_decode_returns(&results.returnData[0].returnData, true)?._0,
            name: nameCall::abi_decode_returns(&results.returnData[1].returnData, true)?._0,
            symbol: symbolCall::abi_decode_returns(&results.returnData[2].returnData, true)?._0,
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ReadContractError(#[from] ReadContractParametersBuilderError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    AbiDecodedErrorType(#[from] AbiDecodedErrorType),
    #[error(transparent)]
    AbiDecodeError(#[from] AbiDecodeFailedErrors),
    #[error(transparent)]
    SolTypesError(#[from] alloy::sol_types::Error),
}
