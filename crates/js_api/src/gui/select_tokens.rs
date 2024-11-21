use std::str::FromStr;

use super::*;

#[wasm_bindgen]
impl DotrainOrderGui {
    pub fn check_token_addresses(&self) -> Result<(), GuiError> {
        for (token, address) in self.select_tokens.iter() {
            if address == &Address::ZERO {
                return Err(GuiError::TokenMustBeSelected(token.clone()));
            }
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = "getSelectTokens")]
    pub fn get_select_tokens(&self) -> Vec<String> {
        self.select_tokens.keys().cloned().collect()
    }

    #[wasm_bindgen(js_name = "saveSelectTokenAddress")]
    pub fn save_select_token_address(
        &mut self,
        token_name: String,
        address: String,
    ) -> Result<(), GuiError> {
        if !self.deployment.select_tokens.contains(&token_name) {
            return Err(GuiError::TokenNotFound(token_name.clone()));
        }
        let address = Address::from_str(&address)?;
        self.select_tokens.insert(token_name.clone(), address);
        self.dotrain_order
            .update_token_address(token_name, address)?;
        self.refresh_gui_deployment()?;
        Ok(())
    }
}
