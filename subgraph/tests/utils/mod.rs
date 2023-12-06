pub mod deploy;
pub mod rpc_node;
pub mod setup;
pub mod subgraph;

use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet, WalletError},
};
pub use setup::get_provider;
use std::sync::Arc;

pub fn get_wallet(index: u32) -> anyhow::Result<Wallet<SigningKey>, WalletError> {
    // By default sued by the EVM node in docker
    let mnemonic = "test test test test test test test test test test test junk";
    let wallet_builder = MnemonicBuilder::<English>::default().phrase(mnemonic);

    return wallet_builder.clone().index(index)?.build();
}

pub async fn get_client(
    wallet: Option<Wallet<SigningKey>>,
) -> anyhow::Result<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = wallet.unwrap_or(get_wallet(0)?);
    let provider = get_provider().await?;
    let chain_id = provider.get_chainid().await?;
    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));
    Ok(client)
}
