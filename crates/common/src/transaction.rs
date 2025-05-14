use alloy::primitives::{ruint::FromUintError, Address, U256};
use alloy::sol_types::SolCall;
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::client::{LedgerClient, LedgerClientError};
use alloy_ethers_typecast::{
    gas_fee_middleware::GasFeeSpeed,
    transaction::{
        ReadableClientError, ReadableClientHttp, WritableClientError, WriteContractParameters,
        WriteContractParametersBuilder, WriteContractParametersBuilderError,
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WritableTransactionExecuteError {
    #[error(transparent)]
    WritableClient(#[from] WritableClientError),
    #[error(transparent)]
    TransactionArgs(#[from] TransactionArgsError),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    LedgerClient(#[from] LedgerClientError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
}

#[derive(Error, Debug)]
pub enum TransactionArgsError {
    #[error(transparent)]
    BuildParameters(#[from] WriteContractParametersBuilderError),
    #[error(transparent)]
    ChainIdParse(#[from] FromUintError<u64>),
    #[error("Chain ID is required, but set to None")]
    ChainIdNone,
    #[error(transparent)]
    ReadableClient(#[from] ReadableClientError),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    LedgerClient(#[from] LedgerClientError),
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TransactionArgs {
    pub orderbook_address: Address,
    pub derivation_index: Option<usize>,
    pub chain_id: Option<u64>,
    pub rpc_url: String,
    pub max_priority_fee_per_gas: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub gas_fee_speed: Option<GasFeeSpeed>,
}

impl TransactionArgs {
    pub fn try_into_write_contract_parameters<T: SolCall + Clone>(
        &self,
        call: T,
        contract: Address,
    ) -> Result<WriteContractParameters<T>, TransactionArgsError> {
        let params = WriteContractParametersBuilder::default()
            .address(contract)
            .call(call)
            .max_priority_fee_per_gas(self.max_priority_fee_per_gas)
            .max_fee_per_gas(self.max_fee_per_gas)
            .build()?;

        Ok(params)
    }

    pub async fn try_fill_chain_id(&mut self) -> Result<(), TransactionArgsError> {
        if self.chain_id.is_none() {
            let chain_id = ReadableClientHttp::new_from_url(self.rpc_url.clone())?
                .get_chainid()
                .await?;
            let chain_id_u64: u64 = chain_id.try_into()?;

            self.chain_id = Some(chain_id_u64);
        }

        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn try_into_ledger_client(self) -> Result<LedgerClient, TransactionArgsError> {
        match self.chain_id {
            Some(chain_id) => {
                let client = LedgerClient::new(
                    self.derivation_index
                        .map(alloy_ethers_typecast::client::HDPath::LedgerLive),
                    chain_id,
                    self.rpc_url.clone(),
                    self.gas_fee_speed,
                )
                .await?;

                Ok(client)
            }
            None => Err(TransactionArgsError::ChainIdNone),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::address;
    use httpmock::MockServer;

    use super::*;
    use rain_orderbook_bindings::IOrderBookV4::vaultBalanceCall;

    #[test]
    fn test_try_into_write_contract_parameters_ok() {
        let args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpc_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_fee_speed: None,
        };

        let call = vaultBalanceCall {
            owner: Address::ZERO,
            token: Address::ZERO,
            vaultId: U256::ZERO,
        };

        let params = args
            .try_into_write_contract_parameters(call.clone(), Address::ZERO)
            .unwrap();

        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, call);
        assert_eq!(params.max_priority_fee_per_gas, None);
        assert_eq!(params.max_fee_per_gas, None);

        let args = TransactionArgs {
            orderbook_address: address!("123abcdef24Ca5003905aA834De7156C68b2E1d0"),
            derivation_index: Some(0),
            chain_id: Some(1),
            rpc_url: "https://mainnet.infura.io/v3/your-api-key".to_string(),
            max_priority_fee_per_gas: Some(U256::from(100)),
            max_fee_per_gas: Some(U256::from(200)),
            gas_fee_speed: Some(GasFeeSpeed::Fast),
        };

        let call = vaultBalanceCall {
            owner: address!("b20a608c624Ca5003905aA834De7156C68b2E1d0"),
            token: address!("00000000219ab540356cBB839Cbe05303d7705Fa"),
            vaultId: U256::from(123456),
        };

        let params = args
            .try_into_write_contract_parameters(
                call.clone(),
                address!("0000000000000000000000000123456789abcdef"),
            )
            .unwrap();

        assert_eq!(
            params.address,
            address!("0000000000000000000000000123456789abcdef")
        );
        assert_eq!(params.call, call);
        assert_eq!(params.max_priority_fee_per_gas, Some(U256::from(100)));
        assert_eq!(params.max_fee_per_gas, Some(U256::from(200)));
    }

    #[tokio::test]
    async fn test_try_fill_chain_id_ok() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc").body_contains("eth_chainId");

            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{ "jsonrpc": "2.0", "id": 1, "result": "0x1" }"#);
        });

        let mut args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpc_url: server.url("/rpc"),
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_fee_speed: None,
        };

        args.try_fill_chain_id().await.unwrap();
        assert_eq!(args.chain_id, Some(1));

        // the URL is invalid but it shouldn't be used now that chain ID is set
        args.rpc_url = "".to_string();
        args.try_fill_chain_id().await.unwrap();
        assert_eq!(args.chain_id, Some(1));
    }

    #[tokio::test]
    async fn test_try_fill_chain_id_err() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc").body_contains("eth_chainId");
            then.status(500);
        });

        let mut args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpc_url: server.url("/rpc"),
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_fee_speed: None,
        };

        let err = args.try_fill_chain_id().await.unwrap_err();
        assert!(matches!(
            err,
            TransactionArgsError::ReadableClient(ReadableClientError::ReadChainIdError(msg))
            if msg.contains("Deserialization Error: EOF while parsing a value at line 1 column 0. Response: ")
        ));
    }

    #[tokio::test]
    async fn test_try_into_ledger_client_err() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc").body_contains("eth_chainId");
            then.status(200)
                .body(r#"{ "jsonrpc": "2.0", "id": 1, "result": "0x1" }"#);
        });

        let mut args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpc_url: server.url("/rpc"),
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_fee_speed: None,
        };

        let err = args.clone().try_into_ledger_client().await;
        assert!(matches!(err, Err(TransactionArgsError::ChainIdNone)));

        args.try_fill_chain_id().await.unwrap();
        args.rpc_url = "".to_string();
        let result = args.try_into_ledger_client().await;
        // The error is different based on whether you have a Ledger plugged in,
        // hence no pattern matching to avoid breaking the test for devs
        assert!(result.is_err());
    }

    // NOTE: `alloy` ignores all ledger tests so it seems like there is no way
    // to mock a device. hence there is only a test case for a scenario that
    // should fail regardless of whether a ledger is connected or not
}
