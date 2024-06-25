use crate::{
    csv::TryIntoCsv,
    utils::timestamp::{format_bigint_timestamp_display, FormatTimestampDisplayError},
};
use alloy_dyn_abi::SolType;
use alloy_primitives::hex::{encode, hex::decode};
use rain_orderbook_bindings::IOrderBookV4::OrderV3;
use rain_orderbook_subgraph_client::types::orders_list::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderFlattened {
    pub id: String,
    pub timestamp: BigInt,
    pub timestamp_display: String,
    pub owner: Bytes,
    pub order_active: bool,
    pub interpreter: Bytes,
    pub interpreter_store: Bytes,
    pub transaction: String,
    pub valid_inputs_vaults: String,
    pub valid_outputs_vaults: String,
    pub valid_inputs_token_symbols_display: String,
    pub valid_outputs_token_symbols_display: String,
}

impl TryFrom<Order> for OrderFlattened {
    type Error = FormatTimestampDisplayError;

    fn try_from(val: Order) -> Result<Self, Self::Error> {
        let order = OrderV3::abi_decode(&decode(&val.order_bytes.0)?, true)?;
        Ok(Self {
            id: val.id.0,
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            owner: val.owner,
            order_active: val.active,
            interpreter: Bytes(encode(order.evaluable.interpreter.0)),
            interpreter_store: Bytes(encode(order.evaluable.store.0)),
            transaction: val.add_events[0].transaction.id.0,
            valid_inputs_vaults: val
                .inputs
                .clone()
                .into_iter()
                .map(|v| v.vault_id.0)
                .collect::<Vec<String>>()
                .join(", "),
            valid_outputs_vaults: val
                .outputs
                .clone()
                .into_iter()
                .map(|v| v.vault_id.0)
                .collect::<Vec<String>>()
                .join(", "),
            valid_inputs_token_symbols_display: val.valid_inputs.map_or("".into(), |v| {
                v.into_iter()
                    .map(|io| io.token.symbol)
                    .collect::<Vec<String>>()
                    .join(", ")
            }),
            valid_outputs_token_symbols_display: val.valid_outputs.map_or("".into(), |v| {
                v.into_iter()
                    .map(|io| io.token.symbol)
                    .collect::<Vec<String>>()
                    .join(", ")
            }),
        })
    }
}

impl TryIntoCsv<OrderFlattened> for Vec<OrderFlattened> {}
