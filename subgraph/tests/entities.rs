mod generated;
mod utils;
use ethers::{types::Bytes, utils::keccak256};
use utils::{
    deploy::{get_orderbook, get_orderbook_meta},
    get_client, subgraph,
};

#[tokio::main]
#[test]
async fn test_orderbook_entity() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    subgraph::wait().await?;

    let response = subgraph::Query::orderbook(&orderbook.address()).await?;

    // The address that  deploy the OrderBook at initialization
    let deployer_address: ethers::types::H160 = get_client(None).await?.address();

    let ob_meta_hashed = Bytes::from(keccak256(get_orderbook_meta()?));

    assert_eq!(response.id, orderbook.address());
    assert_eq!(response.address, orderbook.address());
    assert_eq!(response.deployer, deployer_address);
    assert_eq!(response.meta, ob_meta_hashed);

    Ok(())
}
