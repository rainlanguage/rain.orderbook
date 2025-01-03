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
        let current_deployment = self.get_current_deployment()?;
        let mut select_tokens = self
            .select_tokens
            .clone()
            .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.contains_key(&token_name) {
            return Err(GuiError::TokenNotFound(token_name.clone()));
        }

        let addr = Address::from_str(&address)?;
        select_tokens.insert(token_name.clone(), addr);
        self.select_tokens = Some(select_tokens);

        let rpc_url = current_deployment
            .deployment
            .order
            .orderbook
            .clone()
            .ok_or(GuiError::OrderbookNotFound)?
            .network
            .rpc
            .clone();
        let erc20 = ERC20::new(rpc_url.clone(), addr);
        let token_info = erc20.token_info(None).await?;
        self.onchain_token_info.insert(addr, token_info);

        OrderbookYaml::from_document(self.document.clone())
            .get_token(&token_name)?
            .update_address(&address)?;

        Ok(())
    }
}
