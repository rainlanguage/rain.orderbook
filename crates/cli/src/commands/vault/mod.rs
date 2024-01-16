mod deposit;
mod withdraw;

use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use deposit::Deposit;
use withdraw::Withdraw;

#[derive(Parser)]
pub enum Vault {
    #[command(about = "Deposit tokens into a Vault")]
    Deposit(Deposit),

    #[command(about = "Withdraw tokens from a Vault")]
    Withdraw(Withdraw),
}

impl Execute for Vault {
    async fn execute(&self) -> Result<()> {
        match self {
            Vault::Deposit(deposit) => deposit.execute().await,
            Vault::Withdraw(withdraw) => withdraw.execute().await,
        }
    }
}
