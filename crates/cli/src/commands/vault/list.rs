use std::{fs::canonicalize, path::PathBuf};

use crate::{
    execute::Execute,
    subgraph::{CliSubgraphArgs, CliPaginationArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::{SubgraphArgs, PaginationArgs};
use rain_orderbook_subgraph_client::{
    types::{flattened::TokenVaultFlattened, vaults_list::TokenVault},
    WriteCsv,
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultListArgs {
    #[arg(long, help = "Write results to a CSV file at the path provided")]
    pub csv_file: Option<PathBuf>,

    #[clap(flatten)]
    pub pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let pagination_args: PaginationArgs = self.pagination_args.clone().into();
        let vaults = subgraph_args
            .to_subgraph_client()
            .await?
            .vaults_list(pagination_args)
            .await?;

        if let Some(csv_file) = self.csv_file.clone() {
            let vaults_flattened: Vec<TokenVaultFlattened> =
                vaults.into_iter().map(|o| o.into()).collect();
            vaults_flattened.write_csv(csv_file.clone())?;
            info!("Saved to CSV at {:?}", canonicalize(csv_file.as_path())?);
        } else {
            let table = build_table(vaults)?;
            info!("\n{}", table);
        }

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
