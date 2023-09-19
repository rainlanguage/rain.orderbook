use crate::{
    orderbook::remove_order::v3::remove_order, subgraph::remove_order::v3::get_remove_order,
    transaction::execute_transaction,
};
use anyhow::anyhow;
use clap::Parser;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::H160,
};
use ethers_signers::{HDPath, Ledger};
use std::str::FromStr;
use tracing::{error, info};

#[derive(Parser, Debug, Clone)]
pub struct RemoveOrder {
    /// address of the orderbook
    #[arg(short, long)]
    pub orderbook: String,

    /// address of the orderbook
    #[arg(short, long)]
    pub subgraph_url: String,

    /// id of the order to remove
    #[arg(short = 'i', long)]
    pub order_id: String,

    /// address index of the wallet to accessed. defualt 0.
    #[arg(long, default_value = "0")]
    pub address_index: Option<usize>,

    /// mumbai rpc url, default read from env varibales
    #[arg(long, env)]
    pub rpc_url: Option<String>,

    /// blocknative api key for gas oracle
    #[arg(long, env)]
    pub blocknative_api_key: Option<String>,
}

pub async fn handle_remove_order(order: RemoveOrder) -> anyhow::Result<()> {
    let rpc_url = match order.rpc_url {
        Some(url) => url,
        None => {
            error!("RPC URL NOT PROVIDED");
            return Err(anyhow!("RPC URL not provided."));
        }
    };
    let orderbook_address = match H160::from_str(&order.orderbook) {
        Ok(address) => address,
        Err(err) => {
            error!("ERROR PARSING ORDERBOOK ADDRESS: {}", err);
            return Err(anyhow!(err));
        }
    };

    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            error!("INVALID RPC URL: {}", err);
            return Err(anyhow!(err));
        }
    };

    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let wallet = match Ledger::new(
        HDPath::Other(format!(
            "{}{}",
            String::from("m/44'/60'/0'/0/"),
            order.address_index.unwrap().to_string()
        )),
        chain_id.clone(),
    )
    .await
    {
        Ok(wallet) => wallet,
        Err(err) => {
            error!("ERROR INSTANTIATING LEDGER WALLET: {}", err);
            return Err(anyhow!(err));
        }
    };

    let order_to_remove = get_remove_order(order.subgraph_url, order.order_id)
        .await
        .unwrap();

    let remove_order_tx = remove_order(
        order_to_remove,
        orderbook_address,
        rpc_url.clone(),
        order.blocknative_api_key,
    )
    .await?;

    info!("Removing Order");
    let _ = execute_transaction(rpc_url.clone(), wallet, remove_order_tx).await?;

    Ok(())
}
