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
        let page: i32 = val.page.saturating_sub(1).into();
        let page_size: i32 = val.page_size.into();

        OrdersListQueryVariables {
            skip: Some(page_size * page),
            first: Some(page_size),
        }
    }
}

impl From<SubgraphPaginationArgs> for VaultsListQueryVariables {
    fn from(val: SubgraphPaginationArgs) -> Self {
        let page: i32 = val.page.saturating_sub(1).into();
        let page_size: i32 = val.page_size.into();

        VaultsListQueryVariables {
            skip: Some(page_size * page),
            first: Some(page_size),
        }
    }
}
