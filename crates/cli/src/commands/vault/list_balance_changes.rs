use std::{fs::canonicalize, path::PathBuf};

use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::{
    types::flattened::{TryIntoFlattenedError, VaultBalanceChangeFlattened},
    PaginationArgs, WriteCsv,
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultListBalanceChanges {
    #[arg(short = 'i', long, help = "ID of the Vault")]
    vault_id: String,

    #[arg(long, help = "Write results to a CSV file at the path provided")]
    pub csv_file: Option<PathBuf>,

    #[clap(flatten)]
    pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultListBalanceChanges {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let pagination_args: PaginationArgs = self.pagination_args.clone().into();
        let vault_balance_changes = subgraph_args
            .to_subgraph_client()
            .await?
            .vault_list_balance_changes(self.vault_id.clone().into(), pagination_args)
            .await?;
        let vault_balance_changes_flattened: Vec<VaultBalanceChangeFlattened> =
            vault_balance_changes
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<VaultBalanceChangeFlattened>, TryIntoFlattenedError>>()?;

        if let Some(csv_file) = self.csv_file.clone() {
            vault_balance_changes_flattened.write_csv(csv_file.clone())?;
            info!("Saved to CSV at {:?}", canonicalize(csv_file.as_path())?);
        } else {
            let table = build_table(vault_balance_changes_flattened)?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(balance_change: Vec<VaultBalanceChangeFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "ID",
            "Changed At",
            "Sender",
            "Balance Change",
            "Change Type",
        ]);

    for balance_change in balance_change.into_iter() {
        table.add_row(vec![
            balance_change.id,
            balance_change.timestamp_display,
            balance_change.sender.0,
            balance_change.amount_display_signed,
            balance_change.change_type_display,
        ]);
    }

    Ok(table)
}
