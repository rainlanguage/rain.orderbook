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
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::gui::{
            tests::{initialize_gui, initialize_gui_with_select_tokens},
            GuiError,
        };
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

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use crate::gui::{DotrainOrderGui, GuiError};
        use alloy::primitives::Address;
        use alloy_ethers_typecast::rpc::Response;
        use httpmock::MockServer;
        use std::str::FromStr;

        #[tokio::test]
        async fn test_save_select_token() {
            let server = MockServer::start_async().await;
            let yaml = format!(
                r#"
gui:
  name: Fixed limit
  description: Fixed limit order strategy
  short-description: Buy WETH with USDC on Base.
  deployments:
    some-deployment:
      name: Select token deployment
      description: Select token deployment description
      deposits:
        - token: token3
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
      select-tokens:
        - key: token3
          name: Token 3
          description: Token 3 description
    normal-deployment:
      name: Normal deployment
      description: Normal deployment description
      deposits:
        - token: token3
      fields:
        - binding: binding-1
          name: Field 1 name
          default: 10
networks:
  some-network:
    rpc: {rpc_url}
    chain-id: 123
    network-id: 123
    currency: ETH
subgraphs:
  some-sg: https://www.some-sg.com
metaboards:
  test: https://metaboard.com
deployers:
  some-deployer:
    network: some-network
    address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
  some-orderbook:
    address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
    network: some-network
    subgraph: some-sg
scenarios:
  some-scenario:
    deployer: some-deployer
    bindings:
      test-binding: 5
    scenarios:
      sub-scenario:
        bindings:
          another-binding: 300
orders:
  some-order:
    deployer: some-deployer
    inputs:
      - token: token3
    outputs:
      - token: token3
deployments:
  some-deployment:
    scenario: some-scenario
    order: some-order
  normal-deployment:
    scenario: some-scenario
    order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
                rpc_url = server.url("/rpc")
            );

            server.mock(|when, then| {
            when.method("POST").path("/rpc").body_contains("0x82ad56cb");
            then.body(Response::new_success(1, "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000").to_json_string().unwrap());
        });

            let mut gui = DotrainOrderGui::new();
            gui.choose_deployment(yaml.to_string(), "some-deployment".to_string(), None)
                .await
                .unwrap();

            let deployment = gui.get_current_deployment().unwrap();
            assert_eq!(deployment.deployment.order.inputs[0].token, None);
            assert_eq!(deployment.deployment.order.outputs[0].token, None);

            gui.save_select_token(
                "token3".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
            )
            .await
            .unwrap();
            assert!(gui.is_select_token_set("token3".to_string()).unwrap());

            let deployment = gui.get_current_deployment().unwrap();
            let token = deployment.deployment.order.inputs[0]
                .token
                .as_ref()
                .unwrap();
            assert_eq!(
                token.address,
                Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
            );
            assert_eq!(token.decimals, Some(6));
            assert_eq!(token.label, Some("Token 1".to_string()));
            assert_eq!(token.symbol, Some("T1".to_string()));

            let err = gui
                .save_select_token(
                    "token4".to_string(),
                    "0x0000000000000000000000000000000000000002".to_string(),
                )
                .await
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                GuiError::TokenNotFound("token4".to_string()).to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "The token 'token4' could not be found in the YAML configuration."
            );

            let mut gui = DotrainOrderGui::new();
            gui.choose_deployment(yaml.to_string(), "normal-deployment".to_string(), None)
                .await
                .unwrap();

            let err = gui
                .save_select_token(
                    "token3".to_string(),
                    "0x0000000000000000000000000000000000000002".to_string(),
                )
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), GuiError::SelectTokensNotSet.to_string());
            assert_eq!(
            err.to_readable_msg(),
            "No tokens have been configured for selection. Please check your YAML configuration."
        );
        }
    }
}
