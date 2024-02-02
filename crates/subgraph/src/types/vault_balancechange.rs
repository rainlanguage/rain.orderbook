use crate::types::vault_balancechanges_list::{VaultDeposit, VaultWithdraw};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultBalanceChange {
    Deposit(VaultDeposit),
    Withdraw(VaultWithdraw),
}
