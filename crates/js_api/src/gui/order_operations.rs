use super::*;
use alloy::primitives::{utils::parse_units, Bytes, U256};
use rain_orderbook_common::{deposit::DepositArgs, transaction::TransactionArgs};
use std::{collections::HashMap, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TokenAllowance {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    allowance: U256,
}
impl_wasm_traits!(TokenAllowance);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ApprovalCalldata {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    calldata: Bytes,
}
impl_wasm_traits!(ApprovalCalldata);

#[wasm_bindgen]
impl DotrainOrderGui {
    /// Check allowances for all inputs and outputs of the order
    ///
    /// Returns a vector of [`TokenAllowance`] objects
    pub async fn check_allowances(
        gui: &DotrainOrderGui,
        owner: String,
    ) -> Result<JsValue, GuiError> {
        let orderbook = gui
            .deployment
            .deployment
            .as_ref()
            .order
            .as_ref()
            .orderbook
            .as_ref()
            .ok_or(GuiError::OrderbookNotFound)?;
        let deposits: HashMap<Address, String> = gui
            .get_deposits()
            .into_iter()
            .map(|d| (d.address, d.amount))
            .collect();

        let all_vaults: Vec<_> = gui
            .deployment
            .deployment
            .order
            .inputs
            .iter()
            .chain(
                gui.deployment
                    .deployment
                    .order
                    .outputs
                    .iter()
                    .filter(|output| !gui.deployment.deployment.order.inputs.contains(output)),
            )
            .cloned()
            .collect();

        let mut results = Vec::new();
        for order_io in all_vaults.iter() {
            let allowance = DepositArgs {
                token: order_io.token.address,
                vault_id: order_io.vault_id.ok_or(GuiError::VaultIdNotFound)?,
                amount: parse_units(
                    &deposits[&order_io.token.address],
                    order_io.token.decimals.unwrap_or(18),
                )?
                .into(),
            }
            .read_allowance(
                Address::from_str(&owner)?,
                TransactionArgs {
                    orderbook_address: orderbook.address,
                    rpc_url: orderbook.network.rpc.to_string(),
                    ..Default::default()
                },
            )
            .await?;
            results.push(TokenAllowance {
                token: order_io.token.address,
                allowance,
            });
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    /// Generate approval calldatas for all deposits
    ///
    /// Returns a vector of [`ApprovalCalldata`] objects
    #[wasm_bindgen(js_name = "generateApprovalCalldatas")]
    pub async fn generate_approval_calldatas(&self) -> Result<JsValue, GuiError> {
        todo!()
    }

    #[wasm_bindgen(js_name = "generateDepositCalldatas")]
    pub async fn generate_deposit_calldatas(&self) -> Result<JsValue, GuiError> {
        todo!()
    }

    #[wasm_bindgen(js_name = "generateAddOrderCalldatas")]
    pub async fn generate_add_order_calldatas(&self) -> Result<JsValue, GuiError> {
        todo!()
    }

    #[wasm_bindgen(js_name = "generateDepositAndAddOrderCalldatas")]
    pub async fn generate_deposit_and_add_order_calldatas(&self) -> Result<JsValue, GuiError> {
        todo!()
    }
}
