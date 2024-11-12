use super::*;
use alloy::primitives::U256;
use alloy_ethers_typecast::transaction::{ReadContractParametersBuilder, ReadableClient};
use rain_orderbook_bindings::IERC20::allowanceCall;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TokenAllowance {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    allowance: U256,
}
impl_wasm_traits!(TokenAllowance);

/// Check allowances for all deposits in the state
///
/// Returns a vector of [`TokenAllowance`] objects
pub async fn check_allowances(
    gui: &DotrainOrderGui,
    rpc_url: String,
    owner: String,
) -> Result<JsValue, GuiError> {
    let readable_client = ReadableClient::new_from_url(rpc_url)?;
    let deposits = gui.get_deposits();
    let orderbook = gui
        .deployment
        .deployment
        .as_ref()
        .order
        .as_ref()
        .orderbook
        .as_ref()
        .ok_or(GuiError::OrderbookNotFound)?;

    let mut results = Vec::new();
    for deposit in deposits.iter() {
        let parameters = ReadContractParametersBuilder::<allowanceCall>::default()
            .address(deposit.address)
            .call(allowanceCall {
                owner: Address::from_str(&owner)?,
                spender: orderbook.address,
            })
            .build()?;
        let res = readable_client.read(parameters).await?;
        results.push(TokenAllowance {
            token: deposit.address,
            allowance: res._0,
        });
    }

    Ok(serde_wasm_bindgen::to_value(&results)?)
}
