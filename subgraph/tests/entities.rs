mod generated;
mod utils;

use utils::deploy::get_orderbook;

#[tokio::main]
#[test]
async fn test_orderbook_entity() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    println!("orderbook: {:?}", orderbook.address());

    Ok(())
}
