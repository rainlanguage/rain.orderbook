use crate::orderbook::withdraw::v3::withdraw_tokens;
use crate::transaction::execute_transaction;
use anyhow::anyhow;
use clap::Parser;
use ethers::types::Address;
use ethers::utils::parse_units;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::U256,
};
use ethers_signers::{HDPath, Ledger};
use std::convert::TryFrom;
use std::str::FromStr;
use tracing::{error, info};

#[derive(Parser, Debug, Clone)]
pub struct Withdraw {
    /// address of the orderbook
    #[arg(short, long)]
    pub orderbook: String,

    /// address of the token to withdraw
    #[arg(short = 't', long)]
    pub token_address: String,

    /// decimals coressponding to the token
    #[arg(short = 'd', long)]
    pub token_decimals: u32,

    /// string representing the amount of tokens to be withdrawn
    /// amount will be denominated according to token_decimals
    #[arg(short, long)]
    pub amount: String,

    /// decimal vault id to withdraw from
    #[arg(short, long)]
    pub vault_id: String,

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

pub async fn handle_withdraw(withdraw: Withdraw) -> anyhow::Result<()> {
    let orderbook_address = match Address::from_str(&withdraw.orderbook) {
        Ok(address) => address,
        Err(err) => {
            error!("ERROR PARSING ORDERBOOK ADDRESS: {}", err);
            return Err(anyhow!(err));
        }
    };

    let token_address = match Address::from_str(&withdraw.token_address) {
        Ok(address) => address,
        Err(err) => {
            error!("ERROR PARSING TOKEN ADDRESS: {}", err);
            return Err(anyhow!(err));
        }
    };

    let token_amount: U256 =
        match parse_units(withdraw.amount.clone(), withdraw.token_decimals.clone()) {
            Ok(amount) => amount.into(),
            Err(err) => {
                error!("INVALID TOKEN AMOUNT: {}", err);
                return Err(anyhow!(err));
            }
        };

    let vault_id = match U256::from_dec_str(&String::from(withdraw.clone().vault_id)) {
        Ok(id) => id,
        Err(err) => {
            error!("INVALID VAULT ID: {}", err);
            return Err(anyhow!(err));
        }
    };

    let rpc_url = match withdraw.rpc_url {
        Some(url) => url,
        None => {
            error!("RPC URL NOT PROVIDED");
            return Err(anyhow!("RPC URL not provided."));
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
            withdraw.address_index.unwrap().to_string()
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

    let wallet_address = wallet.get_address().await.unwrap();

    let withdraw_tx = withdraw_tokens(
        token_address,
        token_amount,
        vault_id,
        orderbook_address,
        rpc_url.clone(),
        wallet_address,
        withdraw.blocknative_api_key,
    )
    .await?;

    info!("Withdrawing tokens from vault.");
    let _ = execute_transaction(rpc_url.clone(), wallet, withdraw_tx).await?;

    Ok(())
}
