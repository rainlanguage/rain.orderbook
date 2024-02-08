use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::{
    types::flattened::TokenVaultFlattened, PaginationArgs, TryIntoCsv,
};
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
        let pagination_args: PaginationArgs = self.pagination_args.clone().into();
        let vaults = subgraph_args
            .to_subgraph_client()
            .await?
            .vaults_list(pagination_args)
            .await?;
        let vaults_flattened: Vec<TokenVaultFlattened> =
            vaults.into_iter().map(|o| o.into()).collect();

        if self.pagination_args.csv {
            let csv_text = vaults_flattened.try_into_csv()?;
            println!("{}", csv_text);
        } else {
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
            format!("{}", vault.owner.0),
            vault.token_symbol.clone(),
            format!("{}", vault.balance_display.0),
        ]);
    }

    Ok(table)
}
