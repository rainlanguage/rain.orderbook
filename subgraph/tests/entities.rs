mod generated;
mod subgraph;
mod utils;

use anyhow::Result;
use ethers::{signers::Signer, types::Bytes, utils::keccak256};
use subgraph::{wait, Query};
use utils::{
    cbor::{decode_rain_meta, RainMapDoc},
    deploy::{get_orderbook, read_orderbook_meta},
};

#[tokio::main]
// #[test]
async fn orderbook_entity_test() -> Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Query the OrderBook entity
    let response = Query::orderbook(orderbook.address())
        .await
        .expect("cannot get the ob query response");

    // This wallet is used to deploy the OrderBook at initialization, so it is the deployer
    let wallet_0 = utils::get_wallet(0);

    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta_hashed = Bytes::from(keccak256(read_orderbook_meta()));

    assert_eq!(response.id, orderbook.address());
    assert_eq!(response.address, orderbook.address());
    assert_eq!(response.deployer, wallet_0.address());
    assert_eq!(response.meta, ob_meta_hashed);

    // // Deploy ExpressionDeployerNP for the config
    // let expression_deployer = deploy_touch_deployer(None)
    //     .await
    //     .expect("cannot deploy expression_deployer");

    // ///////////////////////////////////////////////
    // // Deploy ERC20 token contract (A)command) and convert to Bytes
    // let ob_meta_hashed = Bytes::from(keccak256(read_orderbook_meta()));

    // assert_eq!(response.id, orderbook.address());
    // assert_eq!(response.address, orderbook.address());
    // assert_eq!(response.deployer, wallet_0.address());
    // assert_eq!(response.meta, ob_meta_hashed);
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
    // let rain_doc = Bytes::from_hex("0xff0a89c674ee7874a30052746869735f69735f616e5f6578616d706c65011bffe5ffb4a3ff2cde02706170706c69636174696f6e2f6a736f6e")?;

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
    //     meta: rain_doc.clone(),
    // };

    // // Add the order
    // let add_order_func = orderbook.add_order(config);
    // let tx_add_order = add_order_func.send().await.expect("order not sent");

    // let add_order_data = _get_add_order_event(orderbook, tx_add_order).await;
    // println!("add_order_data: {:?}", add_order_data);

    // // // ///////////////////////////////////////////////

    // let mint_func = token_a.mint(wallet_0.address(), _get_amount_tokens(20, 18));
    // let tx_mint = mint_func.send().await.expect("mint not sent");

    // let mint_data = _get_transfer_event(token_a, tx_mint).await;
    // println!("mint_data: {:?}", mint_data);

    // let _ = is_sugraph_node_init()
    //     .await
    //     .expect("cannot check subgraph node");

    Ok(())
}

#[tokio::main]
// #[test]
async fn rain_meta_v1_entity_test() -> Result<()> {
    // Always checking if OB is deployed, so we attemp to obtaing it
    let _ = get_orderbook().await.expect("cannot get OB");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta = read_orderbook_meta();
    let ob_meta_bytes = Bytes::from(ob_meta.clone());
    let ob_meta_hashed = Bytes::from(keccak256(ob_meta));

    // Query the RainMetaV1 entity
    let response = Query::rain_meta_v1(ob_meta_hashed.clone())
        .await
        .expect("cannot get the rain meta query response");

    assert_eq!(response.id, ob_meta_hashed);
    assert_eq!(response.meta_bytes, ob_meta_bytes);
    // assert_eq!(response.content, ob_meta_bytes);

    println!("response.content: {:?}", response.content);

    Ok(())
}

#[test]
fn aver_test() -> Result<()> {
    // Read meta from root repository (output from nix command) and convert to Bytes
    // let ob_meta = read_orderbook_meta();

    // let  ob_meta = <Bytes as hex::FromHex>::from_hex("0xff0a89c674ee7874A3011BFFE5FFB4A3FF2CDE0052946869735F69735F616E5F6578616D706C8502706170706C69636174696F6E2F6A736F6EA4011BFFE5FFB4A3FF2CDF0052746869735F69735F616E5F6578616D706C6502706170706C69636174696F6E2F63626F720362656E").expect("bad hex");

    let  ob_meta = <Bytes as hex::FromHex>::from_hex("0xff0a89c674ee7874A2011BFFE5FFB4A3FF2CDE0052946869735F69735F616E5F6578616D706C8502706170706C69636174696F6E2F6A736F6EA4011BFFE5FFB4A3FF2CDF0052746869735F69735F616E5F6578616D706C6502706170706C69636174696F6E2F63626F720362656E").expect("bad hex");

    let output: Vec<RainMapDoc> = decode_rain_meta(ob_meta)?;

    println!("output.len: {}", output.len());

    Ok(())
}
