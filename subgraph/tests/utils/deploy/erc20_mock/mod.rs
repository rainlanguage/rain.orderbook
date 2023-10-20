use crate::{
    generated::ERC20Mock,
    utils::{get_provider, get_wallet},
};
use anyhow::Result;
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
};
use std::sync::Arc;

pub async fn deploy_erc20_mock(
    wallet: Option<Wallet<SigningKey>>,
) -> Result<ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let contract = ERC20Mock::deploy(client, ())
        .expect("cannot get the ERC20Mock")
        .send()
        .await
        .expect("failed to send ERC20Mock");

    Ok(contract)
}

impl ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    pub async fn connect(
        &self,
        wallet: &Wallet<SigningKey>,
    ) -> ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
        let provider = get_provider().await.expect("cannot get provider");
        let chain_id = provider.get_chainid().await.expect("cannot get chain id");

        let client = Arc::new(SignerMiddleware::new(
            provider.clone(),
            wallet.clone().with_chain_id(chain_id.as_u64()),
        ));

        ERC20Mock::new(self.address(), client)
    }
}
