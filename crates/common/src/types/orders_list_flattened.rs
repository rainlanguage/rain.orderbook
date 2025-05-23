use crate::types::vault::NO_SYMBOL;
use crate::{csv::TryIntoCsv, utils::timestamp::format_bigint_timestamp_display};
use alloy::dyn_abi::SolType;
use alloy::primitives::hex::{decode, encode};
use rain_orderbook_bindings::IOrderBookV4::OrderV3;
use rain_orderbook_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

pub const LIST_DELIMITER: &str = ", ";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderFlattened {
    pub id: String,
    pub timestamp: SgBigInt,
    pub timestamp_display: String,
    pub owner: SgBytes,
    pub order_active: bool,
    pub interpreter: SgBytes,
    pub interpreter_store: SgBytes,
    pub transaction: String,
    pub valid_inputs_vaults: String,
    pub valid_outputs_vaults: String,
    pub valid_inputs_token_symbols_display: String,
    pub valid_outputs_token_symbols_display: String,
    pub trades: String,
}

impl TryFrom<SgOrder> for OrderFlattened {
    type Error = FlattenError;

    fn try_from(val: SgOrder) -> Result<Self, Self::Error> {
        let order = OrderV3::abi_decode(&decode(&val.order_bytes.0)?, true)?;
        Ok(Self {
            id: val.id.0,
            timestamp: val.timestamp_added.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp_added.0)?,
            owner: val.owner,
            order_active: val.active,
            interpreter: SgBytes(encode(order.evaluable.interpreter.0)),
            interpreter_store: SgBytes(encode(order.evaluable.store.0)),
            transaction: val.add_events[0].clone().transaction.id.0,
            valid_inputs_vaults: val
                .inputs
                .clone()
                .into_iter()
                .map(|v| v.vault_id.0)
                .collect::<Vec<String>>()
                .join(LIST_DELIMITER),
            valid_outputs_vaults: val
                .outputs
                .clone()
                .into_iter()
                .map(|v| v.vault_id.0)
                .collect::<Vec<String>>()
                .join(LIST_DELIMITER),
            valid_inputs_token_symbols_display: val
                .inputs
                .into_iter()
                .map(|vault| vault.token.symbol.unwrap_or(NO_SYMBOL.into()))
                .collect::<Vec<String>>()
                .join(LIST_DELIMITER),
            valid_outputs_token_symbols_display: val
                .outputs
                .into_iter()
                .map(|vault| vault.token.symbol.unwrap_or(NO_SYMBOL.into()))
                .collect::<Vec<String>>()
                .join(LIST_DELIMITER),
            trades: val
                .trades
                .into_iter()
                .map(|trade| trade.id.0)
                .collect::<Vec<String>>()
                .join(LIST_DELIMITER),
        })
    }
}

impl TryIntoCsv<OrderFlattened> for Vec<OrderFlattened> {}
