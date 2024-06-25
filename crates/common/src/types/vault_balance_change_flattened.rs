use crate::{
    csv::TryIntoCsv,
    utils::timestamp::{format_bigint_timestamp_display, FormatTimestampDisplayError},
};
use rain_orderbook_subgraph_client::types::{
    vault_balance_changes_list, vault_balance_changes_list::VaultBalanceChange,
};
use serde::{Deserialize, Serialize};

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
    type Error = FormatTimestampDisplayError;

    fn try_from(val: VaultBalanceChange) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            from: val.transaction.from,
            amount: val.amount_display.clone(),
            amount_display_signed: format!("-{}", val.amount_display.0),
            change_type_display: String::from("Withdraw"),
            balance: val.new_vault_balance.clone(),
        })
    }
}

impl TryIntoCsv<VaultBalanceChangeFlattened> for Vec<VaultBalanceChangeFlattened> {}
