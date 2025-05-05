use super::SubgraphError;
use cynic::Id;
use rain_orderbook_subgraph_client::{
    types::common::SgRemoveOrderWithOrder, OrderbookSubgraphClient,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetTransactionRemoveOrdersResult(
    #[tsify(type = "SgRemoveOrderWithOrder[]")] Vec<SgRemoveOrderWithOrder>,
);
impl_wasm_traits!(GetTransactionRemoveOrdersResult);

/// Internal function to fetch Remove Orders for a given transaction
/// Returns an array of RemoveOrder structs
#[wasm_export(
    js_name = "getTransactionRemoveOrders",
    unchecked_return_type = "GetTransactionRemoveOrdersResult"
)]
pub async fn get_transaction_remove_orders(
    url: &str,
    tx_hash: &str,
) -> Result<GetTransactionRemoveOrdersResult, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(GetTransactionRemoveOrdersResult(
        client.transaction_remove_orders(Id::new(tx_hash)).await?,
    ))
}
