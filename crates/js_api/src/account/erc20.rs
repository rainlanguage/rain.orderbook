// TODO add Erc20Error struct
use super::*;
use alloy::primitives::Address;
use alloy::primitives::U256;
use reqwest::Url;
use std::str::FromStr;
#[derive(Debug, Clone, PartialEq, Tsify, Serialize)]

pub struct GetAccountBalanceArgs {
    #[tsify(type = "string")]
    pub rpc_url: String,
    #[tsify(type = "string")]
    pub account_address: String,
    #[tsify(type = "string")]
    pub token_address: String,
}
impl_wasm_traits!(GetAccountBalanceArgs);

#[wasm_bindgen(js_name = "getAccountBalance")]
pub async fn get_account_balance(
    rpc_url: String,
    account_address: String,
    token_address: String,
) -> Result<U256, AccountError> {
    let account =
        Address::from_str(&account_address).map_err(|_| AccountError::AccountBalanceError)?;
    let address =
        Address::from_str(&token_address).map_err(|_| AccountError::AccountBalanceError)?;

    let url = Url::parse(&rpc_url)
        .map_err(|_| AccountError::NetworkError("Invalid RPC URL".to_string()))?;
    let erc20 = ERC20::new(url, address);

    let balance = erc20
        .balance_of(account)
        .await
        .map_err(|_| AccountError::AccountBalanceError)?;
    Ok(balance)
}
