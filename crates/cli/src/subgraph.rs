use clap::Args;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::types::{
    orders_list::OrdersListQueryVariables, vaults_list::VaultsListQueryVariables,
};

#[derive(Args, Clone)]
pub struct CliSubgraphArgs {
    #[arg(
        short,
        long,
        help = "Url of the hosted Subgraph for this Orderbook deployemnt"
    )]
    pub subgraph_url: String,
}

impl From<CliSubgraphArgs> for SubgraphArgs {
    fn from(val: CliSubgraphArgs) -> Self {
        SubgraphArgs {
            url: val.subgraph_url,
        }
    }
}

#[derive(Args, Clone)]
pub struct CliSubgraphPaginationArgs {
    #[arg(short, long, help = "Page number to query", default_value = "1")]
    pub page: u16,

    #[arg(
        short = 'l',
        long,
        help = "Number of items per page",
        default_value = "25"
    )]
    pub page_size: u16,
}

impl From<CliSubgraphPaginationArgs> for OrdersListQueryVariables {
    fn from(val: CliSubgraphPaginationArgs) -> Self {
        let page: i32 = val.page.saturating_sub(1).into();
        let page_size: i32 = val.page_size.into();

        OrdersListQueryVariables {
            skip: Some(page_size * page),
            first: Some(page_size),
        }
    }
}

impl From<CliSubgraphPaginationArgs> for VaultsListQueryVariables {
    fn from(val: CliSubgraphPaginationArgs) -> Self {
        let page: i32 = val.page.saturating_sub(1).into();
        let page_size: i32 = val.page_size.into();

        VaultsListQueryVariables {
            skip: Some(page_size * page),
            first: Some(page_size),
        }
    }
}
