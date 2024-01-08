use clap::Parser;
use anyhow::Result;
use clap::Args;

#[derive(Parser)]
pub struct Deposit {
    #[clap(flatten)]
    deposit_args: DepositArgs
}

impl Deposit {
    pub async fn execute(self) -> Result<()> {
        let DepositArgs {
            token,
            amount,
            vault_id,
        } = &self.deposit_args; 
        println!("Token: {}, Amount: {}, Vault ID: {}", token, amount, vault_id);
        Ok(())
    }
}

#[derive(Args)]
pub struct DepositArgs {
    #[arg(short, long, help = "The token address in hex format")]
    token: String,

    #[arg(short, long, help = "The amount to deposit")]
    amount: u64,

    #[arg(short, long, help = "The ID of the vault")]
    vault_id: u64,
}