mod generated;
mod utils;

use utils::deploy::get_orderbook;
use utils::subgraph::Query;

#[tokio::main]
#[test]
async fn test_orderbook_entity() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    println!("orderbook: {:?}", orderbook.address());

    println!("waiting sync subgraph...");
    utils::subgraph::wait().await?;
    println!("subgraph sync");

    let resp = Query::orderbook(&orderbook.address()).await?;

    println!("{:#?}", resp);

    Ok(())
}
