use crate::utils::deploy::deploy1820;
use anyhow::Result;
use once_cell::sync::Lazy;
use subgraph_rust_setup_utils::{WalletHandler, RPC};
use tokio::sync::OnceCell;

// Initialize just once
static RPC_PROVIDER: Lazy<OnceCell<RPC>> = Lazy::new(|| OnceCell::new());
static WALLETS_HANDLER: Lazy<WalletHandler> = Lazy::new(|| WalletHandler::default());

async fn provider_node() -> Result<RPC> {
    let rpc_provider = RPC::default();

    // Always checking if the Registry1820 is deployed. Deploy it otherwise
    deploy1820(rpc_provider.get_provider()).await?;

    Ok(rpc_provider)
}

pub async fn get_rpc_provider() -> Result<&'static RPC> {
    let provider_lazy = RPC_PROVIDER
        .get_or_try_init(|| async { provider_node().await })
        .await
        .map_err(|err| err);

    match provider_lazy {
        Ok(provider) => Ok(provider),
        Err(err) => return Err(err),
    }
}

pub fn get_wallets_handler() -> &'static WalletHandler {
    &*WALLETS_HANDLER
}
