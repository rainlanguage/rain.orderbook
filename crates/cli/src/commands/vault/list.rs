use crate::{execute::Execute, subgraph::CliSubgraphCommandArgs};
use anyhow::{anyhow, Result};
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_queries::types::vaults::Vault;

use tracing::debug;
#[derive(Args, Clone)]
pub struct VaultListArgs {}

pub type List = CliSubgraphCommandArgs<VaultListArgs>;

impl Execute for List {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let vaults = subgraph_args.to_subgraph_client().await.vaults().await?;
        debug!("{:#?}", vaults);

        let table = build_table(vaults)?;
        println!("{}", table);

        Ok(())
    }
}

fn build_table(vaults: Vec<Vault>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec!["Vault ID", "Owner", "Token", "Balance"]);

    for vault in vaults.iter() {
        let token_vaults = vault
            .token_vaults
            .clone()
            .ok_or(anyhow!("No TokenVault linked to Vault"))?;

        table.add_row(vec![
            vault.id.inner().into(),
            format!("{}", vault.owner.id.0),
            token_vaults[0].token.symbol.clone(),
            token_vaults[0].balance_display.0.clone(),
        ]);
    }

    Ok(table)
}
