#[cfg(not(target_family = "wasm"))]
use crate::transaction::TransactionArgs;
use crate::transaction::TransactionArgsError;
use alloy::primitives::hex::FromHexError;
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::transaction::WritableClientError;
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::transaction::{WriteTransaction, WriteTransactionStatus};
use rain_orderbook_bindings::IOrderBookV4::removeOrder2Call;
use rain_orderbook_subgraph_client::types::{
    common::SgOrder, order_detail_traits::OrderDetailError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RemoveOrderArgsError {
    #[error(transparent)]
    WritableClientError(#[from] WritableClientError),
    #[error(transparent)]
    TransactionArgs(#[from] TransactionArgsError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    OrderDetailError(#[from] OrderDetailError),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoveOrderArgs {
    pub order: SgOrder,
}

impl From<SgOrder> for RemoveOrderArgs {
    fn from(order: SgOrder) -> Self {
        Self { order }
    }
}

impl TryInto<removeOrder2Call> for RemoveOrderArgs {
    type Error = OrderDetailError;

    fn try_into(self) -> Result<removeOrder2Call, OrderDetailError> {
        Ok(removeOrder2Call {
            order: self.order.try_into()?,
            tasks: vec![],
        })
    }
}

impl RemoveOrderArgs {
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<removeOrder2Call>)>(
        self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), RemoveOrderArgsError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let remove_order_call: removeOrder2Call = self.try_into()?;
        let params = transaction_args
            .try_into_write_contract_parameters(
                remove_order_call,
                transaction_args.orderbook_address,
            )
            .await?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_rm_order_calldata(self) -> Result<Vec<u8>, RemoveOrderArgsError> {
        let remove_order_call: removeOrder2Call = self.try_into()?;
        Ok(remove_order_call.abi_encode())
    }
}
