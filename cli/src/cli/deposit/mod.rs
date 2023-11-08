use crate::orderbook::deposit::v3::deposit_tokens;
use crate::transaction::execute_transaction;
use anyhow::anyhow;
use clap::Parser;
use ethers::types::Address;
use ethers::utils::parse_units;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{H160, U256},
};
use ethers_signers::{HDPath, Ledger};
use std::convert::TryFrom;
use std::str::FromStr;
use tracing::{error, info};

use crate::tokens::approve_tokens;

use serde::Deserialize;
#[derive(Parser, Debug, Clone, Deserialize)]
pub struct Deposit {
    /// address of the orderbook
    #[arg(short, long)]
    pub orderbook: String,

    /// address of the token to deposit
    #[arg(short='t', long, num_args = 1..)]
    pub token_address: String,

    /// decimals coressponding to the token
    #[arg(short='d', long, num_args = 1..)]
    pub token_decimals: u32,

    /// string representing the amount of tokens to be deposited
    /// amount will be deominated according to token_decimals
    #[arg(short, long)]
    pub amount: String,

    /// optional vault id to deposit in (in decimals)
    #[arg(short, long)]
    pub vault_id: Option<String>,

    /// address index of the wallet to accessed. defualt 0.
    #[arg(long, default_value = "0")]
    pub address_index: Option<usize>,

    /// rpc url, default read from env varibales
    #[arg(long, env)]
    pub rpc_url: Option<String>,

    /// blocknative api key for gas oracle
    #[arg(long, env)]
    pub blocknative_api_key: Option<String>,
}

pub async fn handle_deposit(deposit: Deposit) -> anyhow::Result<()> {
    let orderbook_address = match Address::from_str(&deposit.orderbook) {
        Ok(address) => address,
        Err(err) => {
            error!("ERROR PARSING ORDERBOOK ADDRESS: {}", err);
            return Err(anyhow!(err));
        }
    };

    let token_address = match Address::from_str(&deposit.token_address) {
        Ok(address) => address,
        Err(err) => {
            error!("ERROR PARSING TOKEN ADDRESS: {}", err);
            return Err(anyhow!(err));
        }
    };

    let token_amount: U256 =
        match parse_units(deposit.amount.clone(), deposit.token_decimals.clone()) {
            Ok(amount) => amount.into(),
            Err(err) => {
                error!("INVALID TOKEN AMOUNT: {}", err);
                return Err(anyhow!(err));
            }
        };

    let vault_id = match deposit.vault_id.clone() {
        Some(val) => match U256::from_dec_str(&val) {
            Ok(id) => id,
            Err(err) => {
                error!("INVALID VAULT ID: {}", err);
                return Err(anyhow!(err));
            }
        },
        None => U256::from(H160::random().as_bytes()),
    };

    let rpc_url = match deposit.rpc_url {
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
            deposit.address_index.unwrap().to_string()
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

    // Approve token for deposit
    let approve_tx = approve_tokens(
        token_address.clone(),
        token_amount.clone(),
        orderbook_address.clone(),
        rpc_url.clone(),
        wallet_address,
        deposit.blocknative_api_key.clone(),
    )
    .await?;

    // Approve Tokens For deposit
    info!("Approving token for deposit");
    let _ = execute_transaction(rpc_url.clone(), wallet, approve_tx).await?;
    // Tokens approved.

    // Deposit tokens
    let wallet = match Ledger::new(
        HDPath::Other(format!(
            "{}{}",
            String::from("m/44'/60'/0'/0/"),
            deposit.address_index.unwrap().to_string()
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

    let deposit_tx = deposit_tokens(
        token_address,
        token_amount,
        vault_id.clone(),
        orderbook_address,
        rpc_url.clone(),
        deposit.blocknative_api_key,
    )
    .await?;

    let _ = execute_transaction(rpc_url.clone(), wallet, deposit_tx).await?;
    // Tokens Deposited.
    Ok(())
}
