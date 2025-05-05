use super::SubgraphError;
use cynic::Id;
use rain_orderbook_subgraph_client::{types::common::SgAddOrderWithOrder, OrderbookSubgraphClient};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetTransactionAddOrdersResult(
    #[tsify(type = "SgAddOrderWithOrder[]")] Vec<SgAddOrderWithOrder>,
);
impl_wasm_traits!(GetTransactionAddOrdersResult);

/// Internal function to fetch Add Orders for a given transaction
/// Returns an array of AddOrder structs
#[wasm_export(
    js_name = "getTransactionAddOrders",
    unchecked_return_type = "GetTransactionAddOrdersResult"
)]
pub async fn get_transaction_add_orders(
    url: &str,
    tx_hash: &str,
) -> Result<GetTransactionAddOrdersResult, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(GetTransactionAddOrdersResult(
        client.transaction_add_orders(Id::new(tx_hash)).await?,
    ))
}
