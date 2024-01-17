use rain_orderbook_subgraph_queries::OrderbookSubgraphClient;
use url::Url;

#[derive(Clone)]
pub struct SubgraphArgs {
    pub url: Url,
}

impl SubgraphArgs {
    pub async fn to_subgraph_client(&self) -> OrderbookSubgraphClient {
        OrderbookSubgraphClient::new(self.url.clone()).await
    }
}
