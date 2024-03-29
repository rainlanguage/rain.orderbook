use crate::{execute::Execute, subgraph::CliSubgraphArgs};
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::subgraph::SubgraphArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultDetailArgs {
    #[arg(short = 'i', long, help = "ID of the Vault")]
    vault_id: String,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultDetailArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let vault = subgraph_args
            .to_subgraph_client()
            .await?
            .vault_detail(self.vault_id.clone().into())
            .await?;
        info!("{:#?}", vault);

        Ok(())
    }
}
