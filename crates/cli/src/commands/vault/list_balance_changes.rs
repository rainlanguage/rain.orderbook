use std::{fs::canonicalize, path::PathBuf};

use crate::{
    execute::Execute,
    subgraph::{CliSubgraphArgs, CliSubgraphPaginationArgs},
};
use anyhow::{anyhow, Result};
use chrono::{NaiveDateTime, TimeZone, Utc};
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::{SubgraphArgs, SubgraphPaginationArgs};
use rain_orderbook_subgraph_client::{
    types::{flattened::VaultBalanceChangeFlattened, vault_balance_change::VaultBalanceChange},
    WriteCsv,
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultListBalanceChanges {
    #[arg(short = 'i', long, help = "ID of the Vault")]
    vault_id: String,

    #[arg(long, help = "Write results to a CSV file at the path provided")]
    pub csv_file: Option<PathBuf>,

    #[clap(flatten)]
    pagination_args: CliSubgraphPaginationArgs,

    #[clap(flatten)]
    subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultListBalanceChanges {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let pagination_args: SubgraphPaginationArgs = self.pagination_args.clone().into();
        let (skip, first): (Option<i32>, Option<i32>) = pagination_args.into();
        let vault_balance_changes = subgraph_args
            .to_subgraph_client()
            .await?
            .vault_list_balance_changes(self.vault_id.clone().into(), skip, first)
            .await?;

        if let Some(csv_file) = self.csv_file.clone() {
            let vault_balance_changes_flattened: Vec<VaultBalanceChangeFlattened> =
                vault_balance_changes.into_iter().map(|o| o.into()).collect();
            vault_balance_changes_flattened.write_csv(csv_file.clone())?;
            info!("Saved to CSV at {:?}", canonicalize(csv_file.as_path())?);
        } else {
            let table = build_table(vault_balance_changes)?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(balance_change: Vec<VaultBalanceChange>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "ID",
            "Changed At (UTC)",
            "Sender",
            "Balance Change",
            "Change Type",
        ]);

    for balance_change in balance_change.iter() {
        match balance_change {
            VaultBalanceChange::Withdraw(w) => {
                let timestamp_i64 = w.timestamp.0.parse::<i64>()?;
                let timestamp_naive = NaiveDateTime::from_timestamp_opt(timestamp_i64, 0)
                    .ok_or(anyhow!("Failed to parse timestamp into NaiveDateTime"))?;
                let timestamp_utc = Utc.from_utc_datetime(&timestamp_naive);

                table.add_row(vec![
                    format!("{}", w.id.clone().into_inner()),
                    format!("{}", timestamp_utc.format("%Y-%m-%d %H:%M:%S")),
                    format!("{}", w.sender.id.clone().0),
                    format!("-{}", w.amount_display.0),
                    "Withdraw".to_string(),
                ]);
            }
            VaultBalanceChange::Deposit(d) => {
                let timestamp_i64 = d.timestamp.0.parse::<i64>()?;
                let timestamp_naive = NaiveDateTime::from_timestamp_opt(timestamp_i64, 0)
                    .ok_or(anyhow!("Failed to parse timestamp into NaiveDateTime"))?;
                let timestamp_utc = Utc.from_utc_datetime(&timestamp_naive);

                table.add_row(vec![
                    format!("{}", d.id.clone().into_inner()),
                    format!("{}", timestamp_utc.format("%Y-%m-%d %H:%M:%S")),
                    format!("{}", d.sender.id.clone().0),
                    format!("{}", d.amount_display.0),
                    "Deposit".to_string(),
                ]);
            }
        };
    }

    Ok(table)
}
