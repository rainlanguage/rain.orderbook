use anyhow::Result;
use clap::Args;
use clap::FromArgMatches;
use clap::Parser;
use rain_orderbook_subgraph_queries::OrderbookSubgraphClient;
use url::Url;

#[derive(Args, Clone)]
pub struct SubgraphArgs {
    #[arg(
        short,
        long,
        help = "Url of the hosted Subgraph for this Orderbook deployemnt"
    )]
    pub subgraph_url: Url,
}

impl SubgraphArgs {
    pub async fn try_into_subgraph_client(&self) -> Result<OrderbookSubgraphClient> {
        Ok(OrderbookSubgraphClient::new(self.subgraph_url.clone()))
    }
}

#[derive(Parser, Clone)]
pub struct CliSubgraphCommandArgs<T: FromArgMatches + Args> {
    #[clap(flatten)]
    pub cmd_args: T,

    #[clap(flatten)]
    pub subgraph_args: SubgraphArgs,
}
