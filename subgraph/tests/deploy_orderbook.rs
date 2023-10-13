#[path = "./generated/abigen/mod.rs"]
mod abigen;
mod common;
mod generated;
mod utils;

use common::{
    deploy::{deploy, Config},
    wait::wait,
};

use abigen::OrderBook::{EvaluableConfigV2, Io, Order, OrderConfigV2, TakeOrderConfig};
use hex::FromHex;
use utils::deploy::orderbook::deploy_orderbook;
use utils::gen_abigen::_abigen_rust_generation;
use utils::setup::get_provider;
use utils::utils::_get_block_number;

use std::ops::Mul;
use std::path::{self, Path};

use anyhow::Result;
use ethers::{abi::AbiEncode, prelude::*};
use ethers::{
    abi::{encode, Token},
    signers::Signer,
    types::{Address, Bytes, U256},
};

use utils::deploy::{erc20_mock::deploy_erc20_mock, touch_deployer::deploy_touch_deployer};
use utils::events::{get_matched_log, get_transfer_event};
use utils::number::get_amount_tokens;

use std::fs::File;
use std::io::{self, Read};
use std::{env, fs};

#[tokio::main]
#[test]
async fn orderbook_entity_test() -> Result<()> {
    let provider = get_provider().await.expect("cannot get provider");
    let block_number = _get_block_number(provider.clone()).await;

    let orderbook = deploy_orderbook(None)
        .await
        .expect("failed when calling deploy orderbook");

    let sg_config = Config {
        contract_address: orderbook.address().to_string(),
        block_number: block_number.as_u64(),
    };

    let _ = deploy(sg_config).await.expect("cannot deploy sg");

    // let is_sync = wait().await.expect("cannot get SG sync status");
    // println!("Sg sync: {}", is_sync);

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
