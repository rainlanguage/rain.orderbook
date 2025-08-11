use alloy::network::AnyNetwork;
use alloy::primitives::{Address, U256};
use alloy::providers::{MulticallError, Provider};
use alloy_ethers_typecast::ReadContractParametersBuilderError;
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use rain_orderbook_bindings::provider::{mk_read_provider, ReadProvider, ReadProviderError};
use rain_orderbook_bindings::IERC20::IERC20Instance;
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
    pub rpcs: Vec<Url>,
    pub address: Address,
}

impl ERC20 {
    pub fn new(rpcs: Vec<Url>, address: Address) -> Self {
        Self { rpcs, address }
    }

    fn get_instance(&self) -> Result<IERC20Instance<ReadProvider, AnyNetwork>, Error> {
        let provider = mk_read_provider(&self.rpcs)?;
        let erc20 = IERC20Instance::new(self.address, provider);
        Ok(erc20)
    }

    pub async fn decimals(&self) -> Result<u8, Error> {
        let erc20 = self.get_instance()?;
        let decimals = erc20.decimals().call().await;

        match decimals {
            Ok(decimals) => Ok(decimals),
            Err(err) => Err(handle_alloy_err(err, "Decimals reverted").await),
        }
    }

    pub async fn name(&self) -> Result<String, Error> {
        let erc20 = self.get_instance()?;
        let name = erc20.name().call().await;

        match name {
            Ok(name) => Ok(name),
            Err(err) => Err(handle_alloy_err(err, "Name reverted").await),
        }
    }

    pub async fn symbol(&self) -> Result<String, Error> {
        let erc20 = self.get_instance()?;
        let symbol = erc20.symbol().call().await;

        match symbol {
            Ok(symbol) => Ok(symbol),
            Err(err) => Err(handle_alloy_err(err, "Symbol reverted").await),
        }
    }

    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Error> {
        let erc20 = self.get_instance()?;
        let allowance = erc20.allowance(owner, spender).call().await;

        match allowance {
            Ok(allowance) => Ok(allowance),
            Err(err) => Err(handle_alloy_err(err, "Allowance reverted").await),
        }
    }

    pub async fn token_info(&self, multicall_address: Option<Address>) -> Result<TokenInfo, Error> {
        let erc20 = self.get_instance()?;

        let multicaller = if let Some(address) = multicall_address {
            erc20.provider().multicall().address(address)
        } else {
            erc20.provider().multicall()
        };

        let multicall = multicaller
            .add(erc20.decimals())
            .add(erc20.name())
            .add(erc20.symbol());

        match multicall.aggregate().await {
            Ok((decimals, name, symbol)) => Ok(TokenInfo {
                decimals,
                name,
                symbol,
            }),
            Err(MulticallError::CallFailed(bytes)) => {
                let err = AbiDecodedErrorType::selector_registry_abi_decode(bytes.as_ref()).await;
                match err {
                    Ok(err) => Err(Error::AbiDecodedErrorType {
                        msg: "Failed to decode token info".to_string(),
                        source: err,
                    }),
                    Err(e) => Err(Error::AbiDecodeError {
                        msg: "Failed to decode token info".to_string(),
                        source: e,
                    }),
                }
            }
            Err(err) => Err(Error::MulticallError(err)),
        }
    }

    pub async fn get_account_balance(&self, account: Address) -> Result<U256, Error> {
        let erc20 = self.get_instance()?;
        let balance = erc20.balanceOf(account).call().await;

        match balance {
            Ok(balance) => Ok(balance),
            Err(err) => Err(handle_alloy_err(err, "Balance query reverted").await),
        }
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
    #[error(transparent)]
    ReadProviderError(#[from] ReadProviderError),
    #[error("Contract call failed: {0}")]
    ContractCallError(#[from] alloy::contract::Error),
    #[error("Multicall failed: {0}")]
    MulticallError(#[from] MulticallError),
}

async fn handle_alloy_err(err: alloy::contract::Error, msg: &str) -> Error {
    if let Some(revert_data) = err.as_revert_data() {
        let err = AbiDecodedErrorType::selector_registry_abi_decode(revert_data.as_ref()).await;

        match err {
            Ok(err) => {
                return Error::AbiDecodedErrorType {
                    msg: msg.to_string(),
                    source: err,
                };
            }
            Err(e) => {
                return Error::AbiDecodeError {
                    msg: msg.to_string(),
                    source: e,
                };
            }
        }
    }

    Error::ContractCallError(err)
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::{hex, sol_types::SolValue};
    use httpmock::MockServer;
    use rain_orderbook_test_fixtures::LocalEvm;
    use serde_json::json;

    #[tokio::test]
    async fn test_decimals() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x313ce567");
            then.json_body_obj(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x0000000000000000000000000000000000000000000000000000000000000012",
            }));
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );

        let decimals = erc20.decimals().await.unwrap();
        assert_eq!(decimals, 18);
    }

    #[tokio::test]
    async fn test_decimals_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x313ce567");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x1",
            }));
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.decimals().await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x313ce567");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x0000000000000000000000000000000000000000000000000000000000000123",
            }));
        });
        assert!(erc20.decimals().await.is_err());
    }

    #[tokio::test]
    async fn test_name() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x06fdde03");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": &hex::encode_prefixed("Test Token".to_string().abi_encode()).to_string(),
            }));
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        let name = erc20.name().await.unwrap();
        assert_eq!(name, "Test Token");
    }

    #[tokio::test]
    async fn test_name_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x06fdde03");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x1",
            }));
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.name().await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x06fdde03");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x0000000000000000000000000000000000000000000000000000000000000123",
            }));
        });
        assert!(erc20.name().await.is_err());
    }

    #[tokio::test]
    async fn test_symbol() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x95d89b41");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": &hex::encode_prefixed("TEST".to_string().abi_encode()).to_string(),
            }));
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        let symbol = erc20.symbol().await.unwrap();
        assert_eq!(symbol, "TEST");
    }

    #[tokio::test]
    async fn test_symbol_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x95d89b41");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x1",
            }));
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.symbol().await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x95d89b41");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000205445535400000000000000000000000000000000000000000000000000000000",
            }));
        });
        assert!(erc20.symbol().await.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_token_info() {
        let local_evm = LocalEvm::new_with_tokens(1).await;
        let token = local_evm.tokens[0].clone();

        let erc20 = ERC20::new(
            vec![Url::parse(&local_evm.url()).unwrap()],
            *token.address(),
        );
        let token_info = erc20.token_info(None).await.unwrap();

        assert_eq!(token_info.decimals, 18);
        assert_eq!(token_info.name, "Token1");
        assert_eq!(token_info.symbol, "TOKEN1");
    }

    #[tokio::test]
    async fn test_token_info_invalid() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x82ad56cb");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x1",
            }));
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );
        assert!(erc20.token_info(None).await.is_err());

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x82ad56cb");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x00000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000012300000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000"
            }));
        });
        assert!(erc20.token_info(None).await.is_err());
    }

    #[tokio::test]
    async fn test_get_account_balance() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x70a08231");
            then.body(
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x00000000000000000000000000000000000000000000000000000000000003e8"
                })
                .to_string(),
            );
        });

        let erc20 = ERC20::new(
            vec![Url::parse(&server.url("/rpc")).unwrap()],
            Address::ZERO,
        );

        let balance = erc20.get_account_balance(Address::ZERO).await.unwrap();
        assert_eq!(balance, alloy::primitives::U256::from(1000u64));
    }
}
