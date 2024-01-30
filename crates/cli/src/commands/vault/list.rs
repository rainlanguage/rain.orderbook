use crate::{
    execute::Execute,
    subgraph::{CliSubgraphArgs, CliSubgraphPaginationArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::types::vaults_list::TokenVault;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultListArgs {
    #[clap(flatten)]
    pub pagination_args: CliSubgraphPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let vaults = subgraph_args
            .to_subgraph_client()
            .await?
            .vaults_list(self.pagination_args.clone().into())
            .await?;

        let table = build_table(vaults)?;
        info!("\n{}", table);

        Ok(())
    }
}

fn build_table(vaults: Vec<TokenVault>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Owner", "Token", "Balance"]);

    for vault in vaults.iter() {
        table.add_row(vec![
            format!("{}", vault.id.clone().into_inner()),
            format!("{}", vault.owner.id.0),
            vault.token.symbol.clone(),
            vault.balance_display.0.clone(),
        ]);
    }

    Ok(table)
}
