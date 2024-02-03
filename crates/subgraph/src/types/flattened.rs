use super::{
    orders_list, vault_balance_change::VaultBalanceChange, vault_list_balance_changes, vaults_list,
};
use crate::csv::WriteCsv;
use serde::{Deserialize, Serialize};

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
    pub handle_io: bool,
    pub owner: orders_list::Bytes,
    pub order_active: bool,
    pub interpreter: orders_list::Bytes,
    pub interpreter_store: orders_list::Bytes,
    pub transaction: String,
    pub valid_inputs_vaults: String,
    pub valid_outputs_vaults: String,
}

impl From<orders_list::Order> for OrderFlattened {
    fn from(val: orders_list::Order) -> Self {
        Self {
            id: val.id.into_inner(),
            timestamp: val.timestamp,
            handle_io: val.handle_io,
            owner: val.owner.id,
            order_active: val.order_active,
            interpreter: val.interpreter,
            interpreter_store: val.interpreter_store,
            transaction: val.transaction.id.into_inner(),
            valid_inputs_vaults: val
                .valid_inputs
                .map_or("".into(), |v: Vec<orders_list::Io>| {
                    v.into_iter()
                        .map(|io| io.token_vault.id.into_inner())
                        .collect::<Vec<String>>()
                        .join(", ")
                }),
            valid_outputs_vaults: val.valid_outputs.map_or("".into(), |v| {
                v.into_iter()
                    .map(|io| io.token_vault.id.into_inner())
                    .collect::<Vec<String>>()
                    .join(", ")
            }),
        }
    }
}

impl WriteCsv<OrderFlattened> for Vec<OrderFlattened> {}

#[derive(Serialize, Deserialize, Clone)]
pub struct VaultBalanceChangeFlattened {
    pub id: String,
    pub timestamp: vault_list_balance_changes::BigInt,
    pub sender: vault_list_balance_changes::Bytes,
    pub amount: vault_list_balance_changes::BigDecimal,
    pub change_type: String,
    pub balance: vault_list_balance_changes::BigDecimal,
}

impl From<VaultBalanceChange> for VaultBalanceChangeFlattened {
    fn from(val: VaultBalanceChange) -> Self {
        match val {
            VaultBalanceChange::Deposit(v) => Self {
                id: v.id.into_inner(),
                timestamp: v.timestamp,
                sender: v.sender.id,
                amount: v.amount_display,
                change_type: String::from("Deposit"),
                balance: v.token_vault.balance_display,
            },
            VaultBalanceChange::Withdraw(v) => Self {
                id: v.id.into_inner(),
                timestamp: v.timestamp,
                sender: v.sender.id,
                amount: v.amount_display,
                change_type: String::from("Withdraw"),
                balance: v.token_vault.balance_display,
            },
        }
    }
}

impl WriteCsv<VaultBalanceChangeFlattened> for Vec<VaultBalanceChangeFlattened> {}
