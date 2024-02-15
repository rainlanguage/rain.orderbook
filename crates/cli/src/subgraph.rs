use clap::Args;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::PaginationArgs;

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
    #[arg(short, long, help = "Page number to query", default_value = "1", conflicts_with("csv"))]
    pub page: u16,

    #[arg(
        short = 'l',
        long,
        help = "Number of items per page",
        default_value = "25",
        conflicts_with("csv")
    )]
    pub page_size: u16,

    #[arg(long, help = "Output results in CSV format", conflicts_with("page"), conflicts_with("page_size"))]
    pub csv: bool,
}

impl From<CliPaginationArgs> for PaginationArgs {
    fn from(val: CliPaginationArgs) -> Self {
        Self {
            page: val.page,
            page_size: val.page_size,
        }
    }
}
