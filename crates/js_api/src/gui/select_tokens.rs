use super::*;
use rain_orderbook_app_settings::{
    deployment::Deployment, network::Network, order::Order, token::Token,
};
use std::str::FromStr;

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "getSelectTokens")]
    pub fn get_select_tokens(&self) -> Result<Vec<String>, GuiError> {
        let deployment = self.get_current_deployment()?;
        Ok(deployment.select_tokens.unwrap_or(vec![]))
    }

    #[wasm_bindgen(js_name = "isSelectTokenSet")]
    pub fn is_select_token_set(&self, key: String) -> Result<bool, GuiError> {
        Ok(self.dotrain_order.orderbook_yaml().get_token(&key).is_ok())
    }

    #[wasm_bindgen(js_name = "checkSelectTokens")]
    pub fn check_select_tokens(&self) -> Result<(), GuiError> {
        let select_tokens = Gui::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;

        if let Some(select_tokens) = select_tokens {
            for key in select_tokens {
                if self.dotrain_order.orderbook_yaml().get_token(&key).is_err() {
                    return Err(GuiError::TokenMustBeSelected(key.clone()));
                }
            }
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "saveSelectToken")]
    pub async fn save_select_token(
        &mut self,
        key: String,
        address: String,
    ) -> Result<(), GuiError> {
        let select_tokens = Gui::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.contains(&key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        let address = Address::from_str(&address)?;

        let order_key = Deployment::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        let network_key =
            Order::parse_network_key(self.dotrain_order.dotrain_yaml().documents, &order_key)?;
        let rpc_url =
            Network::parse_rpc(self.dotrain_order.dotrain_yaml().documents, &network_key)?;

        let erc20 = ERC20::new(rpc_url.clone(), address);
        let token_info = erc20.token_info(None).await?;

        Token::add_record_to_yaml(
            self.dotrain_order.orderbook_yaml().documents,
            &key,
            &network_key,
            &address.to_string(),
            Some(&token_info.decimals.to_string()),
            Some(&token_info.name),
            Some(&token_info.symbol),
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeSelectToken")]
    pub fn remove_select_token(&mut self, key: String) -> Result<(), GuiError> {
        let select_tokens = Gui::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.contains(&key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        Token::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)?;
        Ok(())
    }
}
