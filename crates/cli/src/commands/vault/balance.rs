use crate::subgraph::{fetch_vault_balance}; // Import subgraph query logic
use anyhow::Result;
use clap::{Args, Subcommand};
use crate::telegram::send_telegram_alert; // Telegram notification

#[derive(Args)]
pub struct VaultArgs {
    pub order_id: String, // Order ID to check balance for
    pub threshold: f64,   // The balance threshold for alert
}

#[derive(Subcommand)]
pub enum Vault {
    CheckBalance(VaultArgs),
}

impl Vault {
    pub async fn execute(self) -> Result<()> {
        match self {
            Vault::CheckBalance(args) => {
                let balance = fetch_vault_balance(&args.order_id).await?;
                println!("Vault balance for order {}: {}", args.order_id, balance);

                if balance < args.threshold {
                    // Send a Telegram alert if the balance is below the threshold
                    send_telegram_alert(&args.order_id, balance).await?;
                }
                Ok(())
            }
        }
    }
}
