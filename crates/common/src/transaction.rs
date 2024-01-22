use alloy_ethers_typecast::{
    client::{LedgerClient, LedgerClientError},
    transaction::{
        WriteContractParameters, WriteContractParametersBuilder,
        WriteContractParametersBuilderError,
    },
};
use alloy_primitives::{hex::FromHexError, Address, U256};
use alloy_sol_types::SolCall;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionArgsError {
    #[error("Parse orderbook address error: {0}")]
    ParseOrderbookAddress(#[from] FromHexError),
    #[error("Build parameters error: {0}")]
    BuildParameters(#[from] WriteContractParametersBuilderError),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionArgs {
    pub orderbook_address: String,
    pub derivation_index: Option<usize>,
    pub chain_id: u64,
    pub rpc_url: String,
    pub max_priority_fee_per_gas: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
}

impl TransactionArgs {
    pub fn to_write_contract_parameters<T: SolCall + Clone>(
        &self,
        call: T,
    ) -> Result<WriteContractParameters<T>, TransactionArgsError> {
        let orderbook_address = self
            .orderbook_address
            .parse::<Address>()
            .map_err(TransactionArgsError::ParseOrderbookAddress)?;

        WriteContractParametersBuilder::default()
            .address(orderbook_address)
            .call(call)
            .max_priority_fee_per_gas(self.max_priority_fee_per_gas)
            .max_fee_per_gas(self.max_fee_per_gas)
            .build()
            .map_err(TransactionArgsError::BuildParameters)
    }

    pub async fn to_ledger_client(self) -> Result<LedgerClient, LedgerClientError> {
        LedgerClient::new(self.derivation_index, self.chain_id, self.rpc_url.clone()).await
    }
}
