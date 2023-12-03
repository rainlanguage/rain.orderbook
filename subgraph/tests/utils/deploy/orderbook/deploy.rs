use crate::{
    generated::{Orderbook, ORDERBOOK_ABI, ORDERBOOK_BYTECODE},
    utils::{deploy::touch_deployer, get_provider, get_wallet},
};
use ethers::{
    abi::Token,
    contract::ContractFactory,
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
};
use std::sync::Arc;

pub async fn deploy_orderbook(
) -> anyhow::Result<Orderbook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = get_wallet(0)?;
    let provider = get_provider().await?;
    let chain_id = provider.get_chainid().await?;

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.clone().with_chain_id(chain_id.as_u64()),
    ));

    // Deploying DISpair
    let expression_deployer = touch_deployer().await?;

    // Obtaining OB Meta bytes
    let meta = std::fs::read("../meta/OrderBook.rain.meta")?;

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

    let contract = deploy_transaction.deploy_tokens(args)?.send().await?;

    Ok(Orderbook::new(contract.address(), client))
}

impl Orderbook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    pub async fn connect(
        &self,
        wallet: &Wallet<SigningKey>,
    ) -> anyhow::Result<Orderbook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
        let provider = get_provider().await?;
        let chain_id = provider.get_chainid().await?;

        let client = Arc::new(SignerMiddleware::new(
            provider.clone(),
            wallet.clone().with_chain_id(chain_id.as_u64()),
        ));

        Ok(Orderbook::new(self.address(), client))
    }
}
