// #[path = "./generated/abigen/mod.rs"]
// mod abigen;
mod common;
mod generated;
mod utils;

use anyhow::Result;
use common::{
    deploy::{deploy, Config},
    query::orderbook::get_orderbook_query,
    wait::wait,
};
use ethers::signers::Signer;
use utils::deploy::orderbook::deploy_orderbook;
use utils::setup::{get_provider, is_sugraph_node_init};
use utils::utils::_get_block_number;
// use utils::gen_abigen::_abigen_rust_generation;

#[tokio::main]
#[test]
async fn orderbook_entity_test() -> Result<()> {
    let _ = is_sugraph_node_init().await.expect("failed sg node init");

    let provider = get_provider().await.expect("cannot get provider");
    let block_number = _get_block_number(provider.clone()).await;
    let wallet_0 = utils::utils::get_wallet(0);

    let orderbook = deploy_orderbook(Some(wallet_0.clone()))
        .await
        .expect("failed when calling deploy orderbook");

    let ob_address = format!("{:?}", orderbook.address());

    let sg_config = Config {
        contract_address: ob_address.clone(),
        block_number: block_number.as_u64(),
    };

    let _ = deploy(sg_config).await.expect("cannot deploy sg");

    let _ = wait().await.expect("cannot get SG sync status");

    let response = get_orderbook_query(&ob_address.clone())
        .await
        .expect("cannot get the ob query response");

    assert_eq!(orderbook.address(), response.id);
    assert_eq!(orderbook.address(), response.address);
    assert_eq!(wallet_0.address(), response.deployer);

    // _abigen_rust_generation();
    // let mut file = File::open("../meta/OrderBook.rain.meta")?;

    // let wallet_0 = utils::utils::get_wallet(0);
    // let wallet_1 = utils::utils::get_wallet(1);

    // // Deploy ExpressionDeployerNP
    // let expression_deployer = deploy_touch_deployer(None)
    //     .await
    //     .expect("cannot deploy expression_deployer");

    // ///////////////////////////////////////////////
    // // Deploy ERC20 token contract (A)
    // let token_a = deploy_erc20_mock(None)
    //     .await
    //     .expect("failed on deploy erc20 token A");

    // // Deploy ERC20 token contract (B)
    // let token_b = deploy_erc20_mock(None)
    //     .await
    //     .expect("failed on deploy erc20 token B");

    // // * Build OrderConfig
    // // Build IO (input)
    // let io_input = Io {
    //     token: token_a.address(),
    //     decimals: token_a.decimals().await.unwrap(),
    //     vault_id: U256::from(0),
    // };

    // // Build IO (output)
    // let io_output = Io {
    //     token: token_b.address(),
    //     decimals: token_b.decimals().await.unwrap(),
    //     vault_id: U256::from(0),
    // };

    // let data_parse = Bytes::from_static(b"_ _ _:block-timestamp() chain-id() block-number();:;");
    // let (bytecode, constants) = expression_deployer
    //     .parse(data_parse.clone())
    //     .await
    //     .expect("cannot get value from parse");

    // // An example rain doc (hardcoded - does not contain any well info. Only rain doc well formed)
    // let rain_doc = Bytes::from_hex("0xffe5ffb4a3ff2cdea30052746869735f69735f616e5f6578616d706c65011bffe5ffb4a3ff2cde02706170706c69636174696f6e2f6a736f6e")?;

    // // Build EvaluableConfigV2
    // let eval_config = EvaluableConfigV2 {
    //     deployer: expression_deployer.address(),
    //     bytecode,
    //     constants,
    // };

    // let config = OrderConfigV2 {
    //     valid_inputs: vec![io_input],
    //     valid_outputs: vec![io_output],
    //     evaluable_config: eval_config,
    //     meta: rain_doc,
    // };
    // ///////////////////////////////////////////////

    // let tx = contract.mint(wallet_0.address(), get_amount_tokens(<amount>, <decimals>)).send().await;
    // let receipt = tx.await;
    // get_transfer_event(contract.clone(), tx).await;

    // let _ = is_sugraph_node_init()
    //     .await
    //     .expect("cannot check subgraph node");

    Ok(())
}
