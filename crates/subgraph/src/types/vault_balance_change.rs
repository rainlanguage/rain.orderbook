use crate::types::vault_balance_changes_list::{VaultDeposit, VaultWithdraw};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

// This MUST match the names in tauri-app/src/lib/types/vaultBalanceChange.ts VaultBalanceChangeType
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum VaultBalanceChange {
    Deposit(VaultDeposit),
    Withdraw(VaultWithdraw),
}
