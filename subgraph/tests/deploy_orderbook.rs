mod generated;
mod utils;

use anyhow::Result;
use ethers::abi::{Address, Bytes, Token};
use ethers::contract::ContractFactory;
use generated::{RAINTERPRETEREXPRESSIONDEPLOYER_ABI, RAINTERPRETEREXPRESSIONDEPLOYER_BYTECODE};
use hex::FromHex;
use utils::{
    deploy::meta_getter::get_authoring_meta,
    deploy::touch_deployer::{
        deploy_touch_deployer, rainterpreter_deploy, rainterpreter_store_deploy,
    },
    setup::{get_provider, is_sugraph_node_init},
    utils::get_wallet,
};

use ethers::{prelude::SignerMiddleware, providers::Middleware, signers::Signer};

use std::{
    fs::File,
    io::{BufReader, Read},
    sync::Arc,
};

#[tokio::main]
#[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    // let rain_address = rainterpreter_deploy(None)
    //     .await
    //     .expect("cannot deploy rainiterpreter");

    // println!("rain_address: {:?}", rain_address);

    // let store_address = rainterpreter_store_deploy(None)
    //     .await
    //     .expect("cannot deploy store_address");
    // println!("store_address: {:?}", store_address);

    // let meta_vec = meta_bytes.clone().to_vec();

    // let aver = &[Token::Tuple(
    //     [
    //         Token::Address(rain_address),
    //         Token::Address(store_address),
    //         Token::Bytes(meta_vec),
    //     ]
    //     .to_vec(),
    // )];

    let wallet = get_wallet(0);
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let check = ContractFactory::new(
        RAINTERPRETEREXPRESSIONDEPLOYER_ABI.clone(),
        RAINTERPRETEREXPRESSIONDEPLOYER_BYTECODE.clone(),
        client,
    );

    let rain_address = Address::random();
    let store_address = Address::random();
    let meta_bytes = get_authoring_meta().await;
    let meta_vec = meta_bytes.to_vec();

    let params = vec![Token::Tuple(vec![
        Token::Address(rain_address),
        Token::Address(store_address),
        Token::Bytes(meta_vec),
    ])];

    let tx_aver = check.deploy_tokens(params).expect("failed deploy tokens");

    let resp = tx_aver.legacy().send().await.expect("failed at send tx");

    // let sg_check = is_sugraph_node_init()
    //     .await
    //     .expect("cannot get nothing from sg");

    // println!("sg_check: {}", sg_check);

    Ok(())
}
