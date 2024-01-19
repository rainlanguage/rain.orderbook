use crate::{execute::Execute, subgraph::CliSubgraphCommandArgs};
use anyhow::Result;
use clap::Args;
use tracing::info;

#[derive(Args, Clone)]
pub struct VaultDetailArgs {
    #[arg(short, long, help = "ID of the Vault")]
    vault_id: String,
}

pub type Detail = CliSubgraphCommandArgs<VaultDetailArgs>;

impl Execute for Detail {
    async fn execute(&self) -> Result<()> {
        let vault = self
            .subgraph_args
            .clone()
            .try_into_subgraph_client()
            .await?
            .vault(self.cmd_args.vault_id.clone().into())
            .await?;
        info!("{:#?}", vault);

        Ok(())
    }
}
