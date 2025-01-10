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
    /// Returns a map of token key to address
    #[wasm_bindgen(js_name = "getSelectTokens")]
    pub fn get_select_tokens(&self) -> Result<SelectTokens, GuiError> {
        let gui = self.get_current_deployment()?;

        if gui.select_tokens.is_none() || self.select_tokens.is_none() {
            return Ok(SelectTokens(BTreeMap::new()));
        }

        Ok(SelectTokens(self.select_tokens.clone().unwrap()))
    }

    #[wasm_bindgen(js_name = "saveSelectTokenAddress")]
    pub async fn save_select_token_address(
        &mut self,
        key: String,
        address: String,
    ) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        let mut select_tokens = self
            .select_tokens
            .clone()
            .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.contains_key(&key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        let address = Address::from_str(&address)?;
        select_tokens.insert(key.clone(), address);
        self.select_tokens = Some(select_tokens);

        let rpc_url = deployment
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

        self.dotrain_order
            .orderbook_yaml()
            .get_token(&key)?
            .update_address(&address.to_string())?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "clearSelectTokenAddress")]
    pub fn clear_select_token_address(&mut self, key: String) -> Result<(), GuiError> {
        let mut select_tokens = self
            .select_tokens
            .clone()
            .ok_or(GuiError::SelectTokensNotSet)?;
        select_tokens.insert(key.clone(), Address::ZERO);
        self.select_tokens = Some(select_tokens);
        Ok(())
    }
}
