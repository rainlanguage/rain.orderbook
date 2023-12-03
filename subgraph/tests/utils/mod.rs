pub mod deploy;
pub mod setup;

use ethers::{
    core::k256::ecdsa::SigningKey,
    signers::{coins_bip39::English, MnemonicBuilder, Wallet, WalletError},
};
pub use setup::get_provider;

pub fn get_wallet(index: u32) -> anyhow::Result<Wallet<SigningKey>, WalletError> {
    // By default sued by the EVM node in docker
    let mnemonic = "test test test test test test test test test test test junk";
    let wallet_builder = MnemonicBuilder::<English>::default().phrase(mnemonic);

    return wallet_builder.clone().index(index)?.build();
}
