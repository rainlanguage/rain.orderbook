use crate::types::vault_list_balance_changes::{VaultDeposit, VaultWithdraw};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum VaultBalanceChange {
    Deposit(VaultDeposit),
    Withdraw(VaultWithdraw),
}
