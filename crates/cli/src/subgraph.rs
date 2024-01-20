use clap::Args;
use clap::FromArgMatches;
use clap::Parser;
use rain_orderbook_common::subgraph::SubgraphArgs;

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

#[derive(Parser, Clone)]
pub struct CliSubgraphCommandArgs<T: FromArgMatches + Args> {
    #[clap(flatten)]
    pub cmd_args: T,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}
