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

#[cfg(test)]
impl DotrainOrderGui {
    pub fn add_record_to_yaml(
        &self,
        key: String,
        network_key: String,
        address: String,
        decimals: String,
        label: String,
        symbol: String,
    ) {
        TokenCfg::add_record_to_yaml(
            self.dotrain_order.orderbook_yaml().documents,
            &key,
            &network_key,
            &address,
            Some(&decimals),
            Some(&label),
            Some(&symbol),
        )
        .unwrap();
    }

    pub fn remove_record_from_yaml(&self, key: String) {
        TokenCfg::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::tests::{initialize_gui, initialize_gui_with_select_tokens};
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_get_select_tokens() {
        let gui = initialize_gui_with_select_tokens().await;
        let select_tokens = gui.get_select_tokens().unwrap();
        assert_eq!(select_tokens.len(), 2);
        assert_eq!(select_tokens[0].key, "token3");
        assert_eq!(select_tokens[1].key, "token4");

        let gui = initialize_gui(None).await;
        let select_tokens = gui.get_select_tokens().unwrap();
        assert_eq!(select_tokens.len(), 0);
    }

    #[wasm_bindgen_test]
    async fn test_is_select_token_set() {
        let gui = initialize_gui_with_select_tokens().await;
        let is_select_token_set = gui.is_select_token_set("token3".to_string()).unwrap();
        assert!(!is_select_token_set);

        gui.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let is_select_token_set = gui.is_select_token_set("token3".to_string()).unwrap();
        assert!(is_select_token_set);
    }

    #[wasm_bindgen_test]
    async fn test_check_select_tokens() {
        let gui = initialize_gui_with_select_tokens().await;

        let err = gui.check_select_tokens().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        gui.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let err = gui.check_select_tokens().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token4".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token4' must be selected to proceed."
        );

        gui.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "18".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );

        assert!(gui.check_select_tokens().is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_get_network_key() {
        let gui = initialize_gui_with_select_tokens().await;
        let network_key = gui.get_network_key().unwrap();
        assert_eq!(network_key, "some-network");
    }

    #[wasm_bindgen_test]
    async fn test_save_select_token() {
        let mut gui = initialize_gui_with_select_tokens().await;
        let err = gui
            .save_select_token(
                "token5".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
            )
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenNotFound("token5".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token5' could not be found in the YAML configuration."
        );
    }

    #[wasm_bindgen_test]
    async fn test_remove_select_token() {
        let mut gui = initialize_gui_with_select_tokens().await;
        gui.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let err = gui.remove_select_token("token5".to_string()).unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenNotFound("token5".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token5' could not be found in the YAML configuration."
        );

        assert!(gui.remove_select_token("token3".to_string()).is_ok());
        assert!(!gui.is_select_token_set("token3".to_string()).unwrap());

        let mut gui = initialize_gui(None).await;
        let err = gui.remove_select_token("token3".to_string()).unwrap_err();
        assert_eq!(err.to_string(), GuiError::SelectTokensNotSet.to_string());
        assert_eq!(
            err.to_readable_msg(),
            "No tokens have been configured for selection. Please check your YAML configuration."
        );
    }

    #[wasm_bindgen_test]
    async fn test_are_all_tokens_selected() {
        let gui = initialize_gui_with_select_tokens().await;

        let are_all_tokens_selected = gui.are_all_tokens_selected().unwrap();
        assert!(!are_all_tokens_selected);

        gui.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let are_all_tokens_selected = gui.are_all_tokens_selected().unwrap();
        assert!(!are_all_tokens_selected);

        gui.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "18".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );

        let are_all_tokens_selected = gui.are_all_tokens_selected().unwrap();
        assert!(are_all_tokens_selected);
    }
}
