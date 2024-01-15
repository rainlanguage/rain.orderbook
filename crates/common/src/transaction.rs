use alloy_ethers_typecast::{client::LedgerClient, request_shim::AlloyTransactionRequest};
use alloy_primitives::{Address, U64};
use alloy_sol_types::SolCall;

pub struct TransactionArgs {
    pub orderbook_address: String,
    pub derivation_index: Option<usize>,
    pub chain_id: u64,
    pub rpc_url: String,
}

impl TransactionArgs {
    pub async fn to_transaction_request_with_call<T: SolCall>(
        &self,
        call: T,
    ) -> anyhow::Result<AlloyTransactionRequest> {
        let tx = AlloyTransactionRequest::default()
            .with_to(self.orderbook_address.parse::<Address>()?)
            .with_data(call.abi_encode().clone())
            .with_chain_id(U64::from(self.chain_id));

        Ok(tx)
    }

    pub async fn to_ledger_client(self) -> anyhow::Result<LedgerClient> {
        LedgerClient::new(self.derivation_index, self.chain_id, self.rpc_url.clone()).await
    }
}
