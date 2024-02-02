use alloy_ethers_typecast::{
    client::{LedgerClient, LedgerClientError},
    gas_fee_middleware::GasFeeSpeed,
    transaction::{
        ReadableClientError, ReadableClientHttp, WriteContractParameters,
        WriteContractParametersBuilder, WriteContractParametersBuilderError,
    },
};
use alloy_primitives::{ruint::FromUintError, Address, U256};
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use thiserror::Error;


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
    #[error(transparent)]
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
    pub gas_fee_speed: Option<GasFeeSpeed>
}

impl TransactionArgs {
    pub async fn try_into_write_contract_parameters<T: SolCall + Clone>(
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

    pub async fn try_into_ledger_client(self) -> Result<LedgerClient, TransactionArgsError> {
        match self.chain_id {
            Some(chain_id) => {
                let client =
                    LedgerClient::new(self.derivation_index, chain_id, self.rpc_url.clone(), self.gas_fee_speed)
                        .await?;

                Ok(client)
            }
            None => Err(TransactionArgsError::ChainIdNone),
        }
    }
}
