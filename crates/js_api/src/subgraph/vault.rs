use cynic::Id;
use rain_orderbook_bindings::wasm_traits::prelude::*;
use rain_orderbook_subgraph_client::types::common::VaultsListFilterArgs;
use rain_orderbook_subgraph_client::{
    MultiOrderbookSubgraphClient, MultiSubgraphArgs, OrderbookSubgraphClient,
    OrderbookSubgraphClientError, PaginationArgs,
};
use reqwest::Url;

/// Fetch all vaults from multiple subgraphs
/// Returns a list of VaultWithSubgraphName structs
#[wasm_bindgen(js_name = "getVaults")]
pub async fn get_vaults(
    subgraphs: Vec<MultiSubgraphArgs>,
    filter_args: VaultsListFilterArgs,
    pagination_args: PaginationArgs,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = MultiOrderbookSubgraphClient::new(subgraphs);
    let vaults = client.vaults_list(filter_args, pagination_args).await?;
    Ok(to_value(&vaults)?)
}

/// Fetch a single vault
/// Returns the Vault struct
#[wasm_bindgen(js_name = "getVault")]
pub async fn get_vault(url: &str, id: &str) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let vault = client.vault_detail(Id::new(id)).await?;
    Ok(to_value(&vault)?)
}
