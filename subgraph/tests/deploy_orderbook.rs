mod generated;
mod utils;

use utils::{deploy::touch_deployer::deploy_touch_deployer, setup::is_sugraph_node_init};

#[tokio::main]
#[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    // Deploy
    let expression_deployer = deploy_touch_deployer(None)
        .await
        .expect("cannot deploy expression_deployer");

    println!("i_interpreter: {:?}", expression_deployer.i_interpreter());
    println!("i_store: {:?}", expression_deployer.i_store());
    println!("expression_deployer: {:?}", expression_deployer.address());

    let _ = is_sugraph_node_init()
        .await
        .expect("cannot check subgraph node");

    Ok(())
}
