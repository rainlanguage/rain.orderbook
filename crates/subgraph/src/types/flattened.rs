use std::num::ParseIntError;

use super::{
    orders_list, vault_balance_change::VaultBalanceChange, vault_list_balance_changes, vaults_list,
};
use crate::utils::format_bigint_timestamp_display;
use crate::{csv::WriteCsv, utils::FormatTimestampDisplayError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TryIntoFlattenedError {
    #[error(transparent)]
    FormatTimestampDisplayError(#[from] FormatTimestampDisplayError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenVaultFlattened {
    pub id: String,
    pub owner: vaults_list::Bytes,
    pub vault_id: vaults_list::BigInt,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: i32,
    pub token_address: String,
    pub balance_display: vaults_list::BigDecimal,
    pub balance: vaults_list::BigInt,
}

impl From<vaults_list::TokenVault> for TokenVaultFlattened {
    fn from(val: vaults_list::TokenVault) -> Self {
        Self {
            id: val.id.into_inner(),
            owner: val.owner.id,
            vault_id: val.vault_id,
            token_name: val.token.name,
            token_symbol: val.token.symbol,
            token_decimals: val.token.decimals,
            token_address: val.token.id.into_inner(),
            balance_display: val.balance_display,
            balance: val.balance,
        }
    }
}

impl WriteCsv<TokenVaultFlattened> for Vec<TokenVaultFlattened> {}

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
    type Error = TryIntoFlattenedError;

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

impl WriteCsv<OrderFlattened> for Vec<OrderFlattened> {}

#[derive(Serialize, Deserialize, Clone)]
pub struct VaultBalanceChangeFlattened {
    pub id: String,
    pub timestamp: vault_list_balance_changes::BigInt,
    pub timestamp_display: String,
    pub sender: vault_list_balance_changes::Bytes,
    pub amount: vault_list_balance_changes::BigDecimal,
    pub amount_display_signed: String,
    pub change_type_display: String,
    pub balance: vault_list_balance_changes::BigDecimal,
}

impl TryFrom<VaultBalanceChange> for VaultBalanceChangeFlattened {
    type Error = TryIntoFlattenedError;

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

impl WriteCsv<VaultBalanceChangeFlattened> for Vec<VaultBalanceChangeFlattened> {}
