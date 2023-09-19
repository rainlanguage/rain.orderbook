use crate::cli::deposit::Deposit;
use crate::orderbook::deposit::v3::deposit_tokens;
use crate::tokens::approve_tokens;
use crate::transaction::execute_transaction;
use actix_web::{post, web, HttpResponse};
use derive_more::{Display, Error};
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

#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", message)]
struct DepositErr {
    message: &'static str,
}

impl actix_web::ResponseError for DepositErr {}

#[post("/api/deposit")]
async fn rpc_deposit(deposit: web::Json<Deposit>) -> Result<HttpResponse, DepositErr> {
    let orderbook_address = match Address::from_str(&deposit.orderbook) {
        Ok(address) => address,
        Err(_) => {
            return Err(DepositErr {
                message: "\n ❌Incorrect orderbook address.",
            });
            // return Err(anyhow!("\n ❌Incorrect orderbook address.")) ;
        }
    };

    let token_address = match Address::from_str(&deposit.token_address) {
        Ok(address) => address,
        Err(_) => {
            return Err(DepositErr {
                message: "\n ❌Incorrect token address.",
            });
            // return Err(anyhow!("\n ❌Incorrect token address.")) ;
        }
    };

    let token_amount: U256 =
        match parse_units(deposit.amount.clone(), deposit.token_decimals.clone()) {
            Ok(amount) => amount.into(),
            Err(_) => {
                return Err(DepositErr {
                    message: "\n ❌Incorrect amount.",
                });
                // return Err(anyhow!("\n ❌Incorrect amount.")) ;
            }
        };

    let vault_id = match deposit.vault_id.clone() {
        Some(val) => {
            match U256::from_dec_str(&val) {
                Ok(id) => id,
                Err(_) => {
                    return Err(DepositErr {
                        message: "\n ❌Invalid vault id.",
                    });
                    // return Err(anyhow!("\n ❌Invalid vault id.")) ;
                }
            }
        }
        None => U256::from(H160::random().as_bytes()),
    };

    let rpc_url = match deposit.rpc_url.clone() {
        Some(url) => url,
        None => {
            error!("RPC URL NOT PROVIDED");
            return Err(DepositErr {
                message: "RPC URL not provided.",
            });
        }
    };

    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            error!("INVALID RPC URL: {}", err);
            return Err(DepositErr {
                message: "INVALID RPC URL",
            });
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
            return Err(DepositErr {
                message: "ERROR INSTANTIATING LEDGER WALLET",
            });
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
    .await
    .unwrap();
    info!("Approving token for deposit");
    let _ = execute_transaction(rpc_url.clone(), wallet, approve_tx).await;
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
            return Err(DepositErr {
                message: "ERROR INSTANTIATING LEDGER WALLET",
            });
        }
    };

    let deposit_tx = deposit_tokens(
        token_address,
        token_amount,
        vault_id,
        orderbook_address,
        rpc_url.clone(),
        deposit.blocknative_api_key.clone(),
    )
    .await
    .unwrap();

    println!("Depositing Tokens Into Vaults");
    let _ = execute_transaction(rpc_url.clone(), wallet, deposit_tx).await;

    Ok(HttpResponse::Ok().finish())
}
