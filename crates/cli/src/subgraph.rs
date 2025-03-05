use clap::Args;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::{
    types::common::{SgBytes, SgOrdersListFilterArgs, SgVaultsListFilterArgs},
    SgPaginationArgs,
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
pub struct CliPaginationArgs {
    #[arg(
        short,
        long,
        help = "Page number to query",
        default_value = "1",
        conflicts_with("csv")
    )]
    pub page: u16,

    #[arg(
        short = 'l',
        long,
        help = "Number of items per page",
        default_value = "25",
        conflicts_with("csv")
    )]
    pub page_size: u16,

    #[arg(
        long,
        help = "Output all items in CSV format (not paginated)",
        conflicts_with("page"),
        conflicts_with("page_size")
    )]
    pub csv: bool,
}

impl From<CliPaginationArgs> for SgPaginationArgs {
    fn from(val: CliPaginationArgs) -> Self {
        Self {
            page: val.page,
            page_size: val.page_size,
        }
    }
}

#[derive(Args, Clone)]
pub struct CliFilterArgs {
    #[arg(
        long,
        help = "Filter orders by owner addresses (comma-separated)",
        value_delimiter = ','
    )]
    pub owners: Vec<String>,

    #[arg(long, help = "Filter orders by active status", default_value = "true")]
    pub active: Option<bool>,

    #[arg(
        long,
        help = "Hide vaults with zero balance (default true)",
        default_value = "true"
    )]
    pub hide_zero_balance: Option<bool>,

    #[arg(long, help = "Filter orders by order hash")]
    pub order_hash: Option<String>,
}

impl From<CliFilterArgs> for SgOrdersListFilterArgs {
    fn from(val: CliFilterArgs) -> Self {
        Self {
            owners: val.owners.into_iter().map(SgBytes).collect(),
            active: val.active,
            order_hash: val.order_hash.map(SgBytes),
        }
    }
}
impl From<CliFilterArgs> for SgVaultsListFilterArgs {
    fn from(val: CliFilterArgs) -> Self {
        Self {
            owners: val.owners.into_iter().map(SgBytes).collect(),
            hide_zero_balance: val.hide_zero_balance.unwrap_or(true),
        }
    }
}
