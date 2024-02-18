use crate::transaction::{TransactionArgs, TransactionArgsError};
use alloy_ethers_typecast::transaction::{
    WritableClientError, WriteTransaction, WriteTransactionStatus,
};
use alloy_primitives::hex::FromHexError;

use rain_orderbook_bindings::IOrderBookV3::removeOrderCall;
use rain_orderbook_subgraph_client::types::{
    order_detail::Order, order_detail_traits::OrderDetailError,
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
    pub order: Order,
}

impl From<Order> for RemoveOrderArgs {
    fn from(order: Order) -> Self {
        Self { order }
    }
}

impl TryInto<removeOrderCall> for RemoveOrderArgs {
    type Error = OrderDetailError;

    fn try_into(self) -> Result<removeOrderCall, OrderDetailError> {
        Ok(removeOrderCall {
            order: self.order.try_into()?,
        })
    }
}

impl RemoveOrderArgs {
    pub async fn execute<S: Fn(WriteTransactionStatus<removeOrderCall>)>(
        self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), RemoveOrderArgsError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let remove_order_call: removeOrderCall = self.try_into()?;
        println!("removeOrder\n{:?}", remove_order_call);
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
}
