use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::{
    csv::TryIntoCsv,
    subgraph::SubgraphArgs,
    types::{FlattenError, TokenVaultFlattened},
};
use rain_orderbook_subgraph_client::PaginationArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultListArgs {
    #[clap(flatten)]
    pub pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let vaults = subgraph_args
                .to_subgraph_client()
                .await?
                .vaults_list_all()
                .await?;
            let vaults_flattened: Vec<TokenVaultFlattened> = vaults
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<TokenVaultFlattened>, FlattenError>>()?;

            let csv_text = vaults_flattened.try_into_csv()?;
            println!("{}", csv_text);
        } else {
            let pagination_args: PaginationArgs = self.pagination_args.clone().into();
            let vaults = subgraph_args
                .to_subgraph_client()
                .await?
                .vaults_list(pagination_args)
                .await?;
            let vaults_flattened: Vec<TokenVaultFlattened> = vaults
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<TokenVaultFlattened>, FlattenError>>()?;

            let table = build_table(vaults_flattened)?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(vaults: Vec<TokenVaultFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Owner", "Token", "Balance"]);

    for vault in vaults.iter() {
        table.add_row(vec![
            vault.id.clone(),
            format!("{}", vault.clone().owner.0),
            vault
                .clone()
                .token_symbol
                .unwrap_or("Unknown".into())
                .clone(),
            format!("{}", vault.balance_display),
        ]);
    }

    Ok(table)
}
