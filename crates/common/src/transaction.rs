use alloy_ethers_typecast::{
    client::{LedgerClient, LedgerClientError},
    transaction::{WriteContractParameters, WriteContractParametersBuilder},
};
use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct TransactionArgs {
    pub orderbook_address: String,
    pub derivation_index: Option<usize>,
    pub chain_id: u64,
    pub rpc_url: String,
    pub max_priority_fee_per_gas: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
}

impl TransactionArgs {
    pub async fn to_write_contract_parameters<T: SolCall + Clone>(
        &self,
        call: T,
    ) -> Result<WriteContractParameters<T>> {
        WriteContractParametersBuilder::default()
            .address(self.orderbook_address.parse::<Address>()?)
            .call(call)
            .max_priority_fee_per_gas(self.max_priority_fee_per_gas)
            .max_fee_per_gas(self.max_fee_per_gas)
            .build()
            .map_err(|e| anyhow!(e))
    }

    pub async fn to_ledger_client(self) -> Result<LedgerClient, LedgerClientError> {
        LedgerClient::new(self.derivation_index, self.chain_id, self.rpc_url.clone()).await
    }
}
