use crate::{
    generated::ERC20Test,
    utils::{get_client, get_provider, get_wallet},
};
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
};
use std::sync::Arc;

pub async fn deploy_erc20(
    wallet: Option<Wallet<SigningKey>>,
) -> anyhow::Result<ERC20Test<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let client = get_client(wallet).await?;
    let contract = ERC20Test::deploy(client, ())?.send().await?;
    Ok(contract)
}

impl ERC20Test<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    pub async fn connect(
        &self,
        wallet: &Wallet<SigningKey>,
    ) -> anyhow::Result<ERC20Test<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
        let client = get_client(Some(wallet.to_owned())).await?;
        Ok(ERC20Test::new(self.address(), client))
    }
}
