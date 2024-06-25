use crate::{
    csv::TryIntoCsv,
    utils::timestamp::{format_bigint_timestamp_display, FormatTimestampDisplayError},
};
use rain_orderbook_subgraph_client::types::{order_takes_list::*, Id};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderTakeFlattened {
    pub id: String,
    pub timestamp: BigInt,
    pub timestamp_display: String,
    pub transaction: Bytes,
    pub sender: Bytes,
    pub order_id: Bytes,
    pub ioratio: BigInt,
    pub input: BigInt,
    pub input_display: String,
    pub input_token_id: Id,
    pub input_token_symbol: String,
    pub input_ioindex: BigInt,
    pub output: BigInt,
    pub output_display: String,
    pub output_token_id: Id,
    pub output_token_symbol: String,
    pub output_ioindex: BigInt,
}

impl TryFrom<Trade> for OrderTakeFlattened {
    type Error = FormatTimestampDisplayError;

    fn try_from(val: Trade) -> Result<Self, Self::Error> {
        Ok(Self {
            id: val.id.0,
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            transaction: val.trade_event.transaction.id,
            sender: val.trade_event.sender,
            order_id: val.order.order_hash,
            ioratio: val.ioratio,
            input: val.input_vault_balance_change.amount,
            input_display: val.input_display,
            input_token_id: val.input_token.id,
            input_token_symbol: val.input_token.symbol,
            input_ioindex: val.input_ioindex,
            output: val.output_vault_balance_change.amount,
            output_display: val.output_display,
            output_token_id: val.output_token.id,
            output_token_symbol: val.output_token.symbol,
            output_ioindex: val.output_ioindex,
        })
    }
}

impl TryIntoCsv<OrderTakeFlattened> for Vec<OrderTakeFlattened> {}
