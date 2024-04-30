use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::{
    csv::TryIntoCsv, subgraph::SubgraphArgs, types::VaultBalanceChangeFlattened,
    utils::timestamp::FormatTimestampDisplayError,
};
use rain_orderbook_subgraph_client::PaginationArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultBalanceChangesList {
    #[arg(short = 'i', long, help = "ID of the Vault")]
    vault_id: String,

    #[clap(flatten)]
    pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultBalanceChangesList {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let vault_balance_changes = subgraph_args
                .to_subgraph_client()
                .await?
                .vault_balance_changes_list_all(self.vault_id.clone().into())
                .await?;
            let vault_balance_changes_flattened: Vec<VaultBalanceChangeFlattened> =
                vault_balance_changes
                    .into_iter()
                    .map(|o| o.try_into())
                    .collect::<Result<Vec<VaultBalanceChangeFlattened>, FormatTimestampDisplayError>>()?;

            let csv_text = vault_balance_changes_flattened.try_into_csv()?;
            println!("{}", csv_text);
        } else {
            let pagination_args: PaginationArgs = self.pagination_args.clone().into();
            let vault_balance_changes = subgraph_args
                .to_subgraph_client()
                .await?
                .vault_balance_changes_list(self.vault_id.clone().into(), pagination_args)
                .await?;
            let vault_balance_changes_flattened: Vec<VaultBalanceChangeFlattened> =
                vault_balance_changes
                    .into_iter()
                    .map(|o| o.try_into())
                    .collect::<Result<Vec<VaultBalanceChangeFlattened>, FormatTimestampDisplayError>>()?;

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
