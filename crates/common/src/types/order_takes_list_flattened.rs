use crate::{csv::TryIntoCsv, utils::timestamp::format_bigint_timestamp_display};
use alloy_primitives::{utils::format_units, I256, U256};
use rain_orderbook_subgraph_client::types::order_takes_list::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderTakeFlattened {
    pub id: String,
    pub timestamp: BigInt,
    pub timestamp_display: String,
    pub transaction: Bytes,
    pub sender: Bytes,
    pub order_id: Bytes,
    pub input: BigInt,
    pub input_display: String,
    pub input_token_id: Bytes,
    pub input_token_symbol: Option<String>,
    pub output: BigInt,
    pub output_display: String,
    pub output_token_id: Bytes,
    pub output_token_symbol: Option<String>,
}

impl TryFrom<Trade> for OrderTakeFlattened {
    type Error = FlattenError;

    fn try_from(val: Trade) -> Result<Self, Self::Error> {
        let timestamp = val.timestamp.clone();
        let input_vault_balance_change = val.input_vault_balance_change.clone();
        let output_vault_balance_change = val.output_vault_balance_change.clone();
        let input_amount = input_vault_balance_change.amount.0.parse::<I256>()?;
        let output_amount = output_vault_balance_change.amount.0.parse::<I256>()?;
        let input_decimals = input_vault_balance_change
            .vault
            .token
            .decimals
            .clone()
            .unwrap_or(BigInt("0".into()))
            .0
            .parse::<u8>()?;
        let output_decimals = output_vault_balance_change
            .vault
            .token
            .decimals
            .clone()
            .unwrap_or(BigInt("0".into()))
            .0
            .parse::<u8>()?;

        Ok(Self {
            id: val.id.0,
            timestamp: timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(timestamp.0)?,
            transaction: val.trade_event.transaction.id,
            sender: val.trade_event.sender,
            order_id: val.order.order_hash,
            input: input_vault_balance_change.amount,
            input_display: format_units(input_amount, input_decimals)?,
            input_token_id: input_vault_balance_change.vault.token.id,
            input_token_symbol: input_vault_balance_change.vault.token.symbol,
            output: output_vault_balance_change.amount,
            output_display: format_units(output_amount, output_decimals)?,
            output_token_id: output_vault_balance_change.vault.token.address,
            output_token_symbol: output_vault_balance_change.vault.token.symbol,
        })
    }
}

impl TryIntoCsv<OrderTakeFlattened> for Vec<OrderTakeFlattened> {}
