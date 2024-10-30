use futures::future::join_all;
use reqwest::Url;
use serde::Deserialize;

use crate::{
    types::common::{OrderWithSubgraphName, OrdersListFilterArgs},
    OrderbookSubgraphClient, OrderbookSubgraphClientError, PaginationArgs,
};

#[derive(Debug, Clone, Deserialize)]
pub struct MultiSubgraphArgs {
    url: Url,
    name: String,
}

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
        filter_args: OrdersListFilterArgs,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<OrderWithSubgraphName>, OrderbookSubgraphClientError> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            let pagination_args = pagination_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let orders = client.orders_list(filter_args, pagination_args).await?;
                let wrapped_orders: Vec<OrderWithSubgraphName> = orders
                    .into_iter()
                    .map(|order| OrderWithSubgraphName {
                        order,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_orders)
            }
        });

        let results = join_all(futures).await;

        let mut all_orders: Vec<OrderWithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flat_map(|result| result)
            .collect();

        all_orders.sort_by(|a, b| {
            let a_timestamp = a.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            let b_timestamp = b.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            b_timestamp.cmp(&a_timestamp)
        });

        Ok(all_orders)
    }
}
