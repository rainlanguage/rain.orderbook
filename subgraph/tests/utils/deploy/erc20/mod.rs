use crate::{generated::ERC20Test, utils::get_client};
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
};

pub async fn deploy_erc20(
    wallet: Option<Wallet<SigningKey>>,
) -> anyhow::Result<ERC20Test<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let client = get_client(wallet).await?;
    Ok(ERC20Test::deploy(client, ())?.send().await?)
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
