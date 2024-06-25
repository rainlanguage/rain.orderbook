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
    pub id: String,
    pub timestamp: vault_balance_changes_list::BigInt,
    pub timestamp_display: String,
    pub sender: vault_balance_changes_list::Bytes,
    pub amount: vault_balance_changes_list::BigInt,
    pub amount_display_signed: String,
    pub change_type_display: String,
    pub balance: vault_balance_changes_list::BigInt,
}

impl TryFrom<VaultBalanceChange> for VaultBalanceChangeFlattened {
    type Error = FormatTimestampDisplayError;

    fn try_from(val: VaultBalanceChange) -> Result<Self, Self::Error> {
        match val {
            VaultBalanceChange::Deposit(v) => Ok(Self {
                id: v.id.into_inner(),
                timestamp: v.timestamp.clone(),
                timestamp_display: format_bigint_timestamp_display(v.timestamp.0)?,
                sender: v.sender.id,
                amount: v.amount_display.clone(),
                amount_display_signed: v.amount_display.0,
                change_type_display: String::from("Deposit"),
                balance: v.token_vault.balance_display,
            }),
            VaultBalanceChange::Withdraw(v) => Ok(Self {
                id: v.id.into_inner(),
                timestamp: v.timestamp.clone(),
                timestamp_display: format_bigint_timestamp_display(v.timestamp.0)?,
                sender: v.sender.id,
                amount: v.amount_display.clone(),
                amount_display_signed: format!("-{}", v.amount_display.0),
                change_type_display: String::from("Withdraw"),
                balance: v.token_vault.balance_display,
            }),
        }
    }
}

impl TryIntoCsv<VaultBalanceChangeFlattened> for Vec<VaultBalanceChangeFlattened> {}
