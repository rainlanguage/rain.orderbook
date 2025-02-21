use crate::{
    types::common::{
        SgOrderWithSubgraphName, SgOrdersListFilterArgs, SgVaultWithSubgraphName,
        SgVaultsListFilterArgs,
    },
    OrderbookSubgraphClient, OrderbookSubgraphClientError, SgPaginationArgs,
};
use futures::future::join_all;
use reqwest::Url;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct MultiSubgraphArgs {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    url: Url,
    name: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(MultiSubgraphArgs);

pub struct MultiOrderbookSubgraphClient {
    subgraphs: Vec<MultiSubgraphArgs>,
}
impl MultiOrderbookSubgraphClient {
    pub fn new(subgraphs: Vec<MultiSubgraphArgs>) -> Self {
        Self { subgraphs }
    }

    fn get_orderbook_subgraph_client(&self, url: Url) -> OrderbookSubgraphClient {
        OrderbookSubgraphClient::new(url)
    }

    pub async fn orders_list(
        &self,
        filter_args: SgOrdersListFilterArgs,
        pagination_args: SgPaginationArgs,
    ) -> Result<Vec<SgOrderWithSubgraphName>, OrderbookSubgraphClientError> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            let pagination_args = pagination_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let orders = client.orders_list(filter_args, pagination_args).await?;
                let wrapped_orders: Vec<SgOrderWithSubgraphName> = orders
                    .into_iter()
                    .map(|order| SgOrderWithSubgraphName {
                        order,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_orders)
            }
        });

        let results = join_all(futures).await;

        let mut all_orders: Vec<SgOrderWithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        all_orders.sort_by(|a, b| {
            let a_timestamp = a.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            let b_timestamp = b.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            b_timestamp.cmp(&a_timestamp)
        });

        Ok(all_orders)
    }

    pub async fn vaults_list(
        &self,
        filter_args: SgVaultsListFilterArgs,
        pagination_args: SgPaginationArgs,
    ) -> Result<Vec<SgVaultWithSubgraphName>, OrderbookSubgraphClientError> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            let pagination_args = pagination_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let vaults = client.vaults_list(filter_args, pagination_args).await?;
                let wrapped_vaults: Vec<SgVaultWithSubgraphName> = vaults
                    .into_iter()
                    .map(|vault| SgVaultWithSubgraphName {
                        vault,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_vaults)
            }
        });

        let results = join_all(futures).await;

        let all_vaults: Vec<SgVaultWithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        Ok(all_vaults)
    }
}
