// TODO add Erc20Error struct
use super::*;
use std::str::FromStr;

#[wasm_bindgen(js_name = "getAccountBalance")]
pub async fn get_account_balance(address: string, address: string) -> Result<U256, erc20Error> {
    let account = Address::from_str(&account)?;
    let address = Address::from_str(&address)?;
    let erc20 = ERC20::new(rpc_url.clone(), address);
    let balance = erc20.balance_of(None).await?;
    Ok(balance)
}
