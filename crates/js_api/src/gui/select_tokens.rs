use std::str::FromStr;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct SelectTokens(#[tsify(type = "Map<string, string>")] BTreeMap<String, Address>);
impl_all_wasm_traits!(SelectTokens);

#[wasm_bindgen]
impl DotrainOrderGui {
    pub fn check_token_addresses(&self) -> Result<(), GuiError> {
        if let Some(select_tokens) = &self.select_tokens {
            for (token, address) in select_tokens.iter() {
                if address == &Address::ZERO {
                    return Err(GuiError::TokenMustBeSelected(token.clone()));
                }
            }
        }
        Ok(())
    }

    /// Get all selected tokens and their addresses
    ///
    /// Returns a map of token name to address
    #[wasm_bindgen(js_name = "getSelectTokens")]
    pub fn get_select_tokens(&self) -> Result<SelectTokens, GuiError> {
        let select_tokens = self
            .select_tokens
            .clone()
            .ok_or(GuiError::SelectTokensNotSet)?;
        Ok(SelectTokens(select_tokens))
    }

    #[wasm_bindgen(js_name = "saveSelectTokenAddress")]
    pub async fn save_select_token_address(
        &mut self,
        token_name: String,
        address: String,
    ) -> Result<(), GuiError> {
        let mut select_tokens = self
            .select_tokens
            .clone()
            .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.contains_key(&token_name) {
            return Err(GuiError::TokenNotFound(token_name.clone()));
        }

        let address = Address::from_str(&address)?;
        select_tokens.insert(token_name.clone(), address);
        self.select_tokens = Some(select_tokens);

        let rpc_url = self
            .deployment
            .deployment
            .order
            .orderbook
            .clone()
            .ok_or(GuiError::OrderbookNotFound)?
            .network
            .rpc
            .clone();
        let erc20 = ERC20::new(rpc_url.clone(), address);
        let token_info = erc20.token_info(None).await?;
        self.onchain_token_info.insert(address, token_info);

        // self.dotrain_order
        //     .update_token_address(token_name, address)?;
        // self.refresh_gui_deployment()?;
        Ok(())
    }
}
