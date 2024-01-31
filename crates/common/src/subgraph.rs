use rain_orderbook_subgraph_client::{
    types::{orders_list::OrdersListQueryVariables, vaults_list::VaultsListQueryVariables},
    OrderbookSubgraphClient,
};
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SubgraphPaginationArgs {
    pub page: u16,
    pub page_size: u16,
}

impl From<SubgraphPaginationArgs> for OrdersListQueryVariables {
    fn from(val: SubgraphPaginationArgs) -> Self {
        let (skip, first): (Option<i32>, Option<i32>) = val.into();

        OrdersListQueryVariables { skip, first }
    }
}

impl From<SubgraphPaginationArgs> for VaultsListQueryVariables {
    fn from(val: SubgraphPaginationArgs) -> Self {
        let (skip, first): (Option<i32>, Option<i32>) = val.into();

        Self { skip, first }
    }
}
impl From<SubgraphPaginationArgs> for (Option<i32>, Option<i32>) {
    fn from(val: SubgraphPaginationArgs) -> Self {
        let page: i32 = val.page.saturating_sub(1).into();
        let page_size: i32 = val.page_size.into();

        (Some(page_size * page), Some(page_size))
    }
}
