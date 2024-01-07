use clap::Args;
use anyhow::Result;

#[derive(Args)]
pub struct DepositArgs {
    #[arg(help = "The token address in hex format")]
    token: String,

    #[arg(help = "The amount to deposit")]
    amount: u64,

    #[arg(help = "The ID of the vault")]
    vault_id: u64,
}

pub async fn deposit(args: DepositArgs) -> Result<()> {
    println!("Token: {}, Amount: {}, Vault ID: {}", args.token, args.amount, args.vault_id);
    Ok(())
}