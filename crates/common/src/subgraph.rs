use rain_orderbook_subgraph_queries::OrderbookSubgraphClient;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SubgraphArgs {
    pub url: String,
}

impl SubgraphArgs {
    pub async fn to_subgraph_client(&self) -> Result<OrderbookSubgraphClient, ParseError> {
        Ok(OrderbookSubgraphClient::new(Url::parse(self.url.as_str())?))
    }
}
