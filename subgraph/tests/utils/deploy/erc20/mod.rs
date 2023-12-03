use crate::{
    generated::ERC20Test,
    utils::{get_provider, get_wallet},
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
    let wallet = wallet.unwrap_or(get_wallet(0)?);
    let provider = get_provider().await?;
    let chain_id = provider.get_chainid().await?;

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let contract = ERC20Test::deploy(client, ())?.send().await?;

    Ok(contract)
}

impl ERC20Test<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    pub async fn connect(
        &self,
        wallet: &Wallet<SigningKey>,
    ) -> anyhow::Result<ERC20Test<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
        let provider = get_provider().await?;
        let chain_id = provider.get_chainid().await?;

        let client = Arc::new(SignerMiddleware::new(
            provider.clone(),
            wallet.clone().with_chain_id(chain_id.as_u64()),
        ));

        Ok(ERC20Test::new(self.address(), client))
    }
}
