use alloy_ethers_typecast::{client::LedgerClient, request_shim::AlloyTransactionRequest};
use alloy_primitives::{Address, U256, U64};
use alloy_sol_types::SolCall;

#[derive(Clone)]
pub struct TransactionArgs {
    pub orderbook_address: String,
    pub derivation_path: Option<usize>,
    pub chain_id: u64,
    pub rpc_url: String,
    pub max_priority_fee_per_gas: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
}

impl TransactionArgs {
    pub async fn to_transaction_request_with_call<T: SolCall>(
        &self,
        call: T,
    ) -> anyhow::Result<AlloyTransactionRequest> {
        let tx = AlloyTransactionRequest::default()
            .with_to(Some(self.orderbook_address.parse::<Address>()?))
            .with_data(Some(call.abi_encode().clone()))
            .with_chain_id(Some(U64::from(self.chain_id)))
            .with_max_priority_fee_per_gas(self.max_priority_fee_per_gas)
            .with_max_fee_per_gas(self.max_fee_per_gas);

        Ok(tx)
    }

    pub async fn to_ledger_client(self) -> anyhow::Result<LedgerClient> {
        LedgerClient::new(self.derivation_path, self.chain_id, self.rpc_url.clone()).await
    }
}
