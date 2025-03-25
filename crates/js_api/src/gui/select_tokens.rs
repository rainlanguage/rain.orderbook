use super::*;
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg, gui::GuiSelectTokensCfg, network::NetworkCfg, order::OrderCfg,
    token::TokenCfg, yaml::YamlParsableHash,
};
use std::str::FromStr;

#[wasm_export]
impl DotrainOrderGui {
    #[wasm_export(
        js_name = "getSelectTokens",
        unchecked_return_type = "GuiSelectTokensCfg[]"
    )]
    pub fn get_select_tokens(&self) -> Result<Vec<GuiSelectTokensCfg>, GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        Ok(select_tokens.unwrap_or(vec![]))
    }

    #[wasm_export(js_name = "isSelectTokenSet", unchecked_return_type = "boolean")]
    pub fn is_select_token_set(&self, key: String) -> Result<bool, GuiError> {
        Ok(self.dotrain_order.orderbook_yaml().get_token(&key).is_ok())
    }

    #[wasm_export(js_name = "checkSelectTokens", unchecked_return_type = "void")]
    pub fn check_select_tokens(&self) -> Result<(), GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;

        if let Some(select_tokens) = select_tokens {
            for select_token in select_tokens {
                if self
                    .dotrain_order
                    .orderbook_yaml()
                    .get_token(&select_token.key)
                    .is_err()
                {
                    return Err(GuiError::TokenMustBeSelected(select_token.key.clone()));
                }
            }
        }

        Ok(())
    }

    #[wasm_export(js_name = "getNetworkKey", unchecked_return_type = "string")]
    pub fn get_network_key(&self) -> Result<String, GuiError> {
        let order_key = DeploymentCfg::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        let network_key =
            OrderCfg::parse_network_key(self.dotrain_order.dotrain_yaml().documents, &order_key)?;
        Ok(network_key)
    }

    #[wasm_export(js_name = "saveSelectToken", unchecked_return_type = "void")]
    pub async fn save_select_token(
        &mut self,
        key: String,
        address: String,
    ) -> Result<(), GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.iter().any(|token| token.key == key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        if TokenCfg::parse_from_yaml(self.dotrain_order.dotrain_yaml().documents, &key, None)
            .is_ok()
        {
            TokenCfg::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)?;
        }

        let address = Address::from_str(&address)?;

        let order_key = DeploymentCfg::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        let network_key =
            OrderCfg::parse_network_key(self.dotrain_order.dotrain_yaml().documents, &order_key)?;
        let rpc_url =
            NetworkCfg::parse_rpc(self.dotrain_order.dotrain_yaml().documents, &network_key)?;

        let erc20 = ERC20::new(rpc_url.clone(), address);
        let token_info = erc20.token_info(None).await?;

        TokenCfg::add_record_to_yaml(
            self.dotrain_order.orderbook_yaml().documents,
            &key,
            &network_key,
            &address.to_string(),
            Some(&token_info.decimals.to_string()),
            Some(&token_info.name),
            Some(&token_info.symbol),
        )?;

        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "removeSelectToken", unchecked_return_type = "void")]
    pub fn remove_select_token(&mut self, key: String) -> Result<(), GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.iter().any(|token| token.key == key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        TokenCfg::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)?;

        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "areAllTokensSelected", unchecked_return_type = "boolean")]
    pub fn are_all_tokens_selected(&self) -> Result<bool, GuiError> {
        let select_tokens = self.get_select_tokens()?;
        for token in select_tokens {
            if !self.is_select_token_set(token.key)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
