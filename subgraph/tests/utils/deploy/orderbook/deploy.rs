use crate::{
    generated::{Orderbook, ORDERBOOK_ABI, ORDERBOOK_BYTECODE},
    utils::{deploy::touch_deployer, get_client},
};
use ethers::{
    abi::Token,
    contract::ContractFactory,
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
};

pub async fn deploy_orderbook(
) -> anyhow::Result<Orderbook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    // Deploying DISpair
    let expression_deployer = touch_deployer().await?;

    // Obtaining OB Meta bytes
    let meta = std::fs::read("../meta/OrderBook.rain.meta")?;
    let args = vec![Token::Tuple(vec![
        Token::Address(expression_deployer.address()),
        Token::Bytes(meta),
    ])];

    let client = get_client(None).await?;

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
        let client = get_client(Some(wallet.to_owned())).await?;
        Ok(Orderbook::new(self.address(), client))
    }
}
