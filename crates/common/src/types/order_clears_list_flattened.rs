use crate::{
    csv::TryIntoCsv,
    utils::timestamp::{format_bigint_timestamp_display, FormatTimestampDisplayError},
};
use rain_orderbook_subgraph_client::types::{order_clears_list, Id};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderClearFlattened {
    pub id: String,
    pub timestamp: order_clears_list::BigInt,
    pub timestamp_display: String,
    pub transaction: Id,
    pub sender: order_clears_list::Bytes,
    pub clearer: order_clears_list::Bytes,

    pub order_a_id: String,
    pub bounty_vault_a_id: String,
    pub bounty_amount_a: Option<order_clears_list::BigDecimal>,
    pub bounty_token_a: String,

    pub order_b_id: String,
    pub bounty_vault_b_id: String,
    pub bounty_amount_b: Option<order_clears_list::BigDecimal>,
    pub bounty_token_b: String,
}

impl TryFrom<order_clears_list::OrderClear> for OrderClearFlattened {
    type Error = FormatTimestampDisplayError;

    fn try_from(val: order_clears_list::OrderClear) -> Result<Self, FormatTimestampDisplayError> {
        Ok(Self {
            id: val.id.into_inner(),
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            transaction: val.transaction.id,
            sender: val.sender.id,
            clearer: val.clearer.id,
            order_a_id: val.order_a.id.into_inner(),

            bounty_vault_a_id: val.bounty.bounty_vault_a.id.into_inner(),
            bounty_amount_a: val.bounty.bounty_amount_adisplay,
            bounty_token_a: val.bounty.bounty_token_a.symbol,

            order_b_id: val.order_b.id.into_inner(),
            bounty_vault_b_id: val.bounty.bounty_vault_b.id.into_inner(),
            bounty_amount_b: val.bounty.bounty_amount_bdisplay,
            bounty_token_b: val.bounty.bounty_token_b.symbol,
        })
    }
}

impl TryIntoCsv<OrderClearFlattened> for Vec<OrderClearFlattened> {}
