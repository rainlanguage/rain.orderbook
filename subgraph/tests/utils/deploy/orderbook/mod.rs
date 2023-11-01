use super::get_expression_deployer;
// use super::touch_deployer::touch_deployer;
use crate::generated::{OrderBook, ORDERBOOK_ABI, ORDERBOOK_BYTECODE};
use crate::utils::{get_provider, get_wallet};
use anyhow::Result;
use ethers::{
    abi::Token,
    contract::ContractFactory,
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
};
use std::{env, fs::File, io::Read, sync::Arc};

mod setup;
pub use setup::get_orderbook;

pub async fn deploy_orderbook(
    wallet: Option<Wallet<SigningKey>>,
) -> Result<OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.clone().with_chain_id(chain_id.as_u64()),
    ));

    // Deploying deployer
    let expression_deployer = get_expression_deployer().await?;
    // let expression_deployer = touch_deployer(Some(wallet.clone()))
    //     .await
    //     .expect("cannot touch deployer (ob)");

    // Obtaining OB Meta bytes
    let meta = read_orderbook_meta();

    let args = vec![Token::Tuple(vec![
        Token::Address(expression_deployer.address()),
        Token::Bytes(meta),
    ])];

    // Obtaining OB deploy transaction
    let deploy_transaction = ContractFactory::new(
        ORDERBOOK_ABI.clone(),
        ORDERBOOK_BYTECODE.clone(),
        client.clone(),
    );

    let contract = deploy_transaction
        .deploy_tokens(args)
        .expect("failed deploy tokens")
        .send()
        .await
        .expect("failed at deployment");

    let orderbook = OrderBook::new(contract.address(), client);

    return Ok(orderbook);
}

impl OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    pub async fn connect(
        &self,
        wallet: &Wallet<SigningKey>,
    ) -> OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
        let provider = get_provider().await.expect("cannot get provider");
        let chain_id = provider.get_chainid().await.expect("cannot get chain id");

        let client = Arc::new(SignerMiddleware::new(
            provider.clone(),
            wallet.clone().with_chain_id(chain_id.as_u64()),
        ));

        OrderBook::new(self.address(), client)
    }
}

pub fn read_orderbook_meta() -> Vec<u8> {
    let meta_directory = env::current_dir()
        .expect("cannot get the current directory")
        .parent()
        .expect("cannot get the parent from current dir")
        .join("meta/OrderBook.rain.meta");

    let mut file = File::open(meta_directory).expect("cannot open the file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("failed on read_to_end");

    return contents;
}
