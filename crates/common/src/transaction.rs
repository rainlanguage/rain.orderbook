use alloy_ethers_typecast::{
    client::{LedgerClient, LedgerClientError},
    transaction::{
        ReadableClientError, ReadableClientHttp, WriteContractParameters,
        WriteContractParametersBuilder, WriteContractParametersBuilderError,
    },
};
use alloy_primitives::{hex::FromHexError, Address, U256};
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionArgsError {
    #[error("Parse orderbook address error: {0}")]
    ParseOrderbookAddress(#[from] FromHexError),
    #[error("Build parameters error: {0}")]
    BuildParameters(#[from] WriteContractParametersBuilderError),
    #[error("Parse Chain ID U256 to u64 error")]
    ChainIdParse,
    #[error("Chain ID is required, but set to None")]
    ChainIdNone,
    #[error("Readable client error: {0}")]
    ReadableClient(#[from] ReadableClientError),
    #[error("Ledger Client Error {0}")]
    LedgerClient(#[from] LedgerClientError),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionArgs {
    pub orderbook_address: Address,
    pub derivation_index: Option<usize>,
    pub chain_id: Option<u64>,
    pub rpc_url: String,
    pub max_priority_fee_per_gas: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
}

impl TransactionArgs {
    pub async fn try_into_write_contract_parameters<T: SolCall + Clone>(
        &self,
        call: T,
    ) -> Result<WriteContractParameters<T>, TransactionArgsError> {
        WriteContractParametersBuilder::default()
            .address(self.orderbook_address)
            .call(call)
            .max_priority_fee_per_gas(self.max_priority_fee_per_gas)
            .max_fee_per_gas(self.max_fee_per_gas)
            .build()
            .map_err(TransactionArgsError::BuildParameters)
    }

    pub async fn try_fill_chain_id(&mut self) -> Result<(), TransactionArgsError> {
        if self.chain_id.is_none() {
            let chain_id = ReadableClientHttp::new_from_url(self.rpc_url.clone())
                .map_err(TransactionArgsError::ReadableClient)?
                .get_chainid()
                .await
                .map_err(TransactionArgsError::ReadableClient)?;
            let chain_id_u64: u64 = chain_id
                .try_into()
                .map_err(|_| TransactionArgsError::ChainIdParse)?;

            self.chain_id = Some(chain_id_u64);
        }

        Ok(())
    }

    pub async fn try_into_ledger_client(self) -> Result<LedgerClient, TransactionArgsError> {
        match self.chain_id {
            Some(chain_id) => {
                LedgerClient::new(self.derivation_index, chain_id, self.rpc_url.clone())
                    .await
                    .map_err(TransactionArgsError::LedgerClient)
            }
            None => Err(TransactionArgsError::ChainIdNone),
        }
    }
}
