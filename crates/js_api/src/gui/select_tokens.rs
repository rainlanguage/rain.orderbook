use super::*;
use rain_orderbook_app_settings::token::Token;
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
        let deployment = self.get_current_deployment()?;

        if let Some(select_tokens) = deployment.select_tokens {
            for key in select_tokens {
                self.dotrain_order
                    .orderbook_yaml()
                    .get_token(&key)
                    .map_err(|_| GuiError::SelectTokensNotSet)?;
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
        let deployment = self.get_current_deployment()?;
        if deployment.select_tokens.is_none() {
            return Err(GuiError::SelectTokensNotSet);
        }
        let select_tokens = deployment.select_tokens.unwrap();
        if !select_tokens.contains(&key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        let address = Address::from_str(&address)?;

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

        Token::add_record_to_yaml(
            self.dotrain_order.orderbook_yaml().documents,
            &key,
            &deployment.deployment.scenario.deployer.network.key,
            &address.to_string(),
            Some(&token_info.decimals.to_string()),
            Some(&token_info.name),
            Some(&token_info.symbol),
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeSelectToken")]
    pub fn remove_select_token(&mut self, key: String) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        if deployment.select_tokens.is_none() {
            return Err(GuiError::SelectTokensNotSet);
        }
        let select_tokens = deployment.select_tokens.unwrap();
        if !select_tokens.contains(&key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        Token::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)?;
        Ok(())
    }
}
