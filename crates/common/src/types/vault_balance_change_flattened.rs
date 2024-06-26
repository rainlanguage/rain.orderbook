use crate::{csv::TryIntoCsv, utils::timestamp::format_bigint_timestamp_display};
use alloy_primitives::{utils::format_units, U256};
use rain_orderbook_subgraph_client::types::vault_balance_changes_list::{
    self, BigInt, VaultBalanceChange,
};
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone)]
pub struct VaultBalanceChangeFlattened {
    pub timestamp: vault_balance_changes_list::BigInt,
    pub timestamp_display: String,
    pub from: vault_balance_changes_list::Bytes,
    pub amount: vault_balance_changes_list::BigInt,
    pub amount_display_signed: String,
    pub change_type_display: String,
    pub balance: vault_balance_changes_list::BigInt,
}

impl TryFrom<VaultBalanceChange> for VaultBalanceChangeFlattened {
    type Error = FlattenError;

    fn try_from(val: VaultBalanceChange) -> Result<Self, Self::Error> {
        let amount_display = format_units(
            val.amount.0.parse::<U256>()?,
            val.vault
                .token
                .decimals
                .unwrap_or(BigInt("0".into()))
                .0
                .parse::<u8>()?,
        )?;
        // if val.new_vault_balance < val.old_vault_balance then it's a negative sign
        let sign = if val.new_vault_balance.0.parse::<U256>()?
            < val.old_vault_balance.0.parse::<U256>()?
        {
            "-"
        } else {
            ""
        };

        Ok(Self {
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            from: val.transaction.from,
            amount: val.amount,
            amount_display_signed: format!("-{}{}", sign, amount_display),
            change_type_display: val.__typename,
            balance: val.new_vault_balance.clone(),
        })
    }
}

impl TryIntoCsv<VaultBalanceChangeFlattened> for Vec<VaultBalanceChangeFlattened> {}
