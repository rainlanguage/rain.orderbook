use ethers::abi::{Bytes, Token, Tokenizable};
use ethers::etherscan::contract::SourceCodeLanguage;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Http, Middleware};
use ethers::signers::coins_bip39::{English, Mnemonic};
use ethers::signers::{MnemonicBuilder, Wallet};
// use ethers::types::{Bytes, Eip1559TransactionRequest, H160, U256};
use ethers::types::{Eip1559TransactionRequest, H160, U256};
use ethers::utils::Anvil;
use ethers::{
    abi,
    core::k256::ecdsa::SigningKey,
    prelude::abigen,
    providers::Provider,
    signers::{LocalWallet, Signer},
    utils::AnvilInstance,
};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::FromStr;
use std::time::Duration;

use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::utils::setup::get_provider;
use crate::utils::utils::get_wallet;

use super::meta_getter::get_authoring_meta;
abigen!(
    RainterpreterExpressionDeployer,
    "tests/utils/deploy/touch_deployer/RainterpreterExpressionDeployer.json",
    derives(serde::Deserialize, serde::Serialize);

    Rainterpreter,
    "tests/utils/deploy/touch_deployer/Rainterpreter.json";

    RainterpreterStore,
    "tests/utils/deploy/touch_deployer/RainterpreterStore.json";
);

pub async fn deploy_touch_deployer(wallet: Option<Wallet<SigningKey>>) -> anyhow::Result<H160> {
    let wallet = Some(wallet.unwrap_or(get_wallet(0)));

    let rainterpreter_address = rainterpreter_deploy(wallet.clone()).await?;
    println!("rainterpreter_address: {:?}", rainterpreter_address);

    let store_address = rainterpreter_store_deploy(wallet.clone()).await?;
    println!("store_address: {:?}", store_address);

    let expression_deployer =
        rainterpreter_expression_deployer_deploy(rainterpreter_address, store_address, None)
            .await
            .expect("failed at expression_deployer_deploy");
    println!("expression_deployer: {:?}", expression_deployer);

    Ok(expression_deployer)
}

pub async fn rainterpreter_deploy(wallet: Option<Wallet<SigningKey>>) -> anyhow::Result<H160> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let store = Rainterpreter::deploy(deployer, ())?.send().await?;
    Ok(store.address())
}

pub async fn rainterpreter_store_deploy(
    wallet: Option<Wallet<SigningKey>>,
) -> anyhow::Result<H160> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let store = RainterpreterStore::deploy(deployer, ())?.send().await?;
    Ok(store.address())
}

pub async fn rainterpreter_expression_deployer_deploy(
    rainiterpreter_address: H160,
    store_address: H160,
    wallet: Option<Wallet<SigningKey>>,
) -> anyhow::Result<H160> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let meta_bytes = get_authoring_meta().await;

    let mut args: Vec<Token> = Vec::new();

    args.push(Token::Address(rainiterpreter_address));
    args.push(Token::Address(store_address));
    // args.push(Token::Bytes(meta_bytes));

    let see = [Token::Tuple(
        [
            Token::Address(rainiterpreter_address),
            Token::Address(store_address),
            Token::Bytes(meta_bytes.to_vec()),
        ]
        .to_vec(),
    )];

    // let aver = ethabi::encode(&[Token::Tuple(
    //     [
    //         Token::Address(rainiterpreter_address),
    //         Token::Address(store_address),
    //         Token::Bytes(meta_bytes.to_vec()),
    //     ]
    //     .to_vec(),
    // )]);

    // // println!("Avec u8: {:?}", aver);
    // let xddd = aver
    //     .iter()
    //     .map(|byte| format!("{:02x}", byte))
    //     .collect::<Vec<String>>()
    //     .join("");

    // let see = [[
    //     rainiterpreter_address.as_bytes(),
    //     store_address.as_bytes(),
    //     &meta_bytes.to_vec(),
    // ]];
    // //////////////////////

    // let file =
    //     File::open("tests/utils/deploy/touch_deployer/RainterpreterExpressionDeployer_ABI.json")?;
    // let reader = BufReader::new(file);

    // let contract_a = abi::Abi::load(reader).expect("cannot get abi");

    // // // let aver = meta_bytes.bytes();
    // let aver = meta_bytes.to_vec();

    // let check_const = contract_a
    //     .constructor()
    //     .expect("xd")
    //     .encode_input(
    //         Bytes::from_hex("ff").expect("cannot create bytes"),
    //         &[Token::Tuple(
    //             [
    //                 Token::Address(rain_address),
    //                 Token::Address(store_address),
    //                 Token::Bytes(aver),
    //             ]
    //             .to_vec(),
    //         )],
    //     )
    //     .expect("failed at construction encode");

    // ////////////////////////

    let expression_deployer = RainterpreterExpressionDeployer::deploy(deployer, args)
        .expect("failed at deploy() expression deployer")
        .send()
        .await
        .expect("failed after send() expression deployer");

    Ok(expression_deployer.address())
}
