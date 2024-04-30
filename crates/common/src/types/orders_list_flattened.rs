use crate::{
    csv::TryIntoCsv,
    utils::timestamp::{format_bigint_timestamp_display, FormatTimestampDisplayError},
};
use rain_orderbook_subgraph_client::types::orders_list;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderFlattened {
    pub id: String,
    pub timestamp: orders_list::BigInt,
    pub timestamp_display: String,
    pub handle_io: bool,
    pub owner: orders_list::Bytes,
    pub order_active: bool,
    pub interpreter: orders_list::Bytes,
    pub interpreter_store: orders_list::Bytes,
    pub transaction: String,
    pub valid_inputs_vaults: String,
    pub valid_outputs_vaults: String,
    pub valid_inputs_token_symbols_display: String,
    pub valid_outputs_token_symbols_display: String,
}

impl TryFrom<orders_list::Order> for OrderFlattened {
    type Error = FormatTimestampDisplayError;

    fn try_from(val: orders_list::Order) -> Result<Self, Self::Error> {
        Ok(Self {
            id: val.id.into_inner(),
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            handle_io: val.handle_io,
            owner: val.owner.id,
            order_active: val.order_active,
            interpreter: val.interpreter,
            interpreter_store: val.interpreter_store,
            transaction: val.transaction.id.into_inner(),
            valid_inputs_vaults: val.valid_inputs.clone().map_or(
                "".into(),
                |v: Vec<orders_list::Io>| {
                    v.into_iter()
                        .map(|io| io.token_vault.id.into_inner())
                        .collect::<Vec<String>>()
                        .join(", ")
                },
            ),
            valid_outputs_vaults: val.valid_outputs.clone().map_or("".into(), |v| {
                v.into_iter()
                    .map(|io| io.token_vault.id.into_inner())
                    .collect::<Vec<String>>()
                    .join(", ")
            }),
            valid_inputs_token_symbols_display: val.valid_inputs.map_or(
                "".into(),
                |v: Vec<orders_list::Io>| {
                    v.into_iter()
                        .map(|io| io.token.symbol)
                        .collect::<Vec<String>>()
                        .join(", ")
                },
            ),
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
