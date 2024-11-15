mod deposit;
mod detail;
mod list;
mod list_balance_changes;
mod withdraw;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use deposit::CliVaultDepositArgs;
use detail::CliVaultDetailArgs;
use list::CliVaultListArgs;
use list_balance_changes::CliVaultBalanceChangesList;
use withdraw::CliVaultWithdrawArgs;

#[derive(Parser)]
pub enum Vault {
    #[command(about = "Deposit tokens into a Vault")]
    Deposit(CliVaultDepositArgs),

    #[command(about = "Withdraw tokens from a Vault")]
    Withdraw(CliVaultWithdrawArgs),

    #[command(about = "List all Vaults", alias = "ls")]
    List(CliVaultListArgs),

    #[command(about = "View a Vault", alias = "view")]
    Detail(CliVaultDetailArgs),

    #[command(about = "List balance changes for a Vault (i.e. deposits + withdraws)")]
    ListBalanceChanges(CliVaultBalanceChangesList),
}

impl Execute for Vault {
    async fn execute(&self) -> Result<()> {
        match self {
            Vault::Deposit(deposit) => deposit.execute().await,
            Vault::Withdraw(withdraw) => withdraw.execute().await,
            Vault::List(list) => list.execute().await,
            Vault::Detail(detail) => detail.execute().await,
            Vault::ListBalanceChanges(list_balance_changes) => list_balance_changes.execute().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_command() {
        Vault::command().debug_assert();
    }
}
