use crate::{execute::Execute, subgraph::CliSubgraphCommandArgs};
use anyhow::Result;
use clap::Args;
use rain_orderbook_common::subgraph::SubgraphArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultDetailArgs {
    #[arg(short, long, help = "ID of the Vault")]
    vault_id: String,
}

pub type Detail = CliSubgraphCommandArgs<CliVaultDetailArgs>;

impl Execute for Detail {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let vault = subgraph_args
            .to_subgraph_client()
            .await?
            .vault(self.cmd_args.vault_id.clone().into())
            .await?;
        info!("{:#?}", vault);

        Ok(())
    }
}
