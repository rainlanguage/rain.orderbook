use crate::{
    csv::TryIntoCsv,
    utils::timestamp::{format_bigint_timestamp_display, FormatTimestampDisplayError},
};
use rain_orderbook_subgraph_client::types::{order_takes_list, Id};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderTakeFlattened {
    pub id: String,
    pub timestamp: order_takes_list::BigInt,
    pub timestamp_display: String,
    pub transaction: Id,
    pub sender: order_takes_list::Bytes,
    pub order_id: Id,
    pub ioratio: order_takes_list::BigInt,
    pub input: order_takes_list::BigInt,
    pub input_display: String,
    pub input_token_id: Id,
    pub input_token_symbol: String,
    pub input_ioindex: order_takes_list::BigInt,
    pub output: order_takes_list::BigInt,
    pub output_display: String,
    pub output_token_id: Id,
    pub output_token_symbol: String,
    pub output_ioindex: order_takes_list::BigInt,
}

impl TryFrom<order_takes_list::Trade> for OrderTakeFlattened {
    type Error = FormatTimestampDisplayError;

    fn try_from(val: order_takes_list::Trade) -> Result<Self, Self::Error> {
        Ok(Self {
            id: val.id.into_inner(),
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            transaction: val.transaction.id,
            sender: val.sender.id,
            order_id: val.order.id,
            ioratio: val.ioratio,
            input: val.input,
            input_display: val.input_display,
            input_token_id: val.input_token.id,
            input_token_symbol: val.input_token.symbol,
            input_ioindex: val.input_ioindex,
            output: val.output,
            output_display: val.output_display,
            output_token_id: val.output_token.id,
            output_token_symbol: val.output_token.symbol,
            output_ioindex: val.output_ioindex,
        })
    }
}

impl TryIntoCsv<OrderTakeFlattened> for Vec<OrderTakeFlattened> {}
