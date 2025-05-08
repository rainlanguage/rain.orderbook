use super::*;
use rain_orderbook_app_settings::gui::GuiDepositCfg;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TokenDeposit {
    pub token: String,
    pub amount: String,
    #[tsify(type = "string")]
    pub address: Address,
}

impl DotrainOrderGui {
    pub fn check_deposits(&self) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;

        for deposit in deployment.deposits.iter() {
            if deposit.token.is_none() {
                return Err(GuiError::MissingDepositToken(deployment.key.clone()));
            }

            let token = deposit.token.as_ref().unwrap();
            if !self.deposits.contains_key(&token.key) {
                return Err(GuiError::DepositNotSet(
                    token
                        .symbol
                        .clone()
                        .unwrap_or(token.label.clone().unwrap_or(token.key.clone())),
                ));
            }
        }
        Ok(())
    }
}

#[wasm_export]
impl DotrainOrderGui {
    fn get_gui_deposit(&self, key: &str) -> Result<GuiDepositCfg, GuiError> {
        let deployment = self.get_current_deployment()?;
        let gui_deposit = deployment
            .deposits
            .iter()
            .find(|dg| dg.token.as_ref().map_or(false, |t| t.key == *key))
            .ok_or(GuiError::DepositTokenNotFound(key.to_string()))?;
        Ok(gui_deposit.clone())
    }

    #[wasm_export(js_name = "getDeposits", unchecked_return_type = "TokenDeposit[]")]
    pub fn get_deposits(&self) -> Result<Vec<TokenDeposit>, GuiError> {
        self.deposits
            .iter()
            .map(|(key, value)| {
                let gui_deposit = self.get_gui_deposit(key)?;
                let amount: String = if value.is_preset {
                    let index = value
                        .value
                        .parse::<usize>()
                        .map_err(|_| GuiError::InvalidPreset)?;
                    gui_deposit
                        .presets
                        .as_ref()
                        .ok_or(GuiError::PresetsNotSet)?
                        .get(index)
                        .ok_or(GuiError::InvalidPreset)?
                        .clone()
                } else {
                    value.value.clone()
                };

                if gui_deposit.token.is_none() {
                    return Err(GuiError::TokenMustBeSelected(key.clone()));
                }
                let token = gui_deposit.token.as_ref().unwrap();

                Ok(TokenDeposit {
                    token: token.key.clone(),
                    amount,
                    address: token.address,
                })
            })
            .collect::<Result<Vec<TokenDeposit>, GuiError>>()
    }

    #[wasm_export(js_name = "saveDeposit", unchecked_return_type = "void")]
    pub fn save_deposit(&mut self, token: String, amount: String) -> Result<(), GuiError> {
        let gui_deposit = self.get_gui_deposit(&token)?;

        if amount.is_empty() {
            self.remove_deposit(token)?;
            return Ok(());
        }

        let value = match gui_deposit.presets.as_ref() {
            Some(presets) => match presets.iter().position(|p| **p == amount) {
                Some(index) => field_values::PairValue {
                    is_preset: true,
                    value: index.to_string(),
                },
                None => field_values::PairValue {
                    is_preset: false,
                    value: amount,
                },
            },
            None => field_values::PairValue {
                is_preset: false,
                value: amount,
            },
        };

        self.deposits.insert(token, value);

        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "removeDeposit", unchecked_return_type = "void")]
    pub fn remove_deposit(&mut self, token: String) -> Result<(), GuiError> {
        self.deposits.remove(&token);
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "getDepositPresets", unchecked_return_type = "string[]")]
    pub fn get_deposit_presets(&self, key: String) -> Result<Vec<String>, GuiError> {
        let gui_deposit = self.get_gui_deposit(&key)?;
        Ok(gui_deposit.presets.clone().unwrap_or(vec![]))
    }

    #[wasm_export(js_name = "getMissingDeposits", unchecked_return_type = "string[]")]
    pub fn get_missing_deposits(&self) -> Result<Vec<String>, GuiError> {
        let deployment = self.get_current_deployment()?;
        let mut missing_deposits = Vec::new();

        for deposit in deployment.deposits.iter() {
            if let Some(token) = &deposit.token {
                if !self.deposits.contains_key(&token.key) {
                    missing_deposits.push(token.key.clone());
                }
            }
        }
        Ok(missing_deposits)
    }

    #[wasm_export(js_name = "hasAnyDeposit", unchecked_return_type = "boolean")]
    pub fn has_any_deposit(&self) -> Result<bool, GuiError> {
        Ok(!self.deposits.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::tests::{initialize_gui, initialize_gui_with_select_tokens};
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_get_gui_deposit() {
        let gui = initialize_gui(None).await;

        let deposit = gui.get_gui_deposit("token1").unwrap();
        assert_eq!(deposit.token.unwrap().key, "token1");
        assert_eq!(
            deposit.presets,
            Some(vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ])
        );

        let err = gui.get_gui_deposit("token2").unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositTokenNotFound("token2".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit token 'token2' was not found in the YAML configuration."
        );

        let gui = initialize_gui_with_select_tokens().await;
        let err = gui.get_gui_deposit("token3").unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositTokenNotFound("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit token 'token3' was not found in the YAML configuration."
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_deposits() {
        let mut gui = initialize_gui(None).await;

        gui.save_deposit("token1".to_string(), "999".to_string())
            .unwrap();

        let deposit = gui.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);
        assert_eq!(deposit[0].token, "token1");
        assert_eq!(deposit[0].amount, "999");
        assert_eq!(
            deposit[0].address,
            Address::from_str("0xc2132d05d31c914a87c6611c10748aeb04b58e8f").unwrap()
        );
    }

    #[wasm_bindgen_test]
    async fn test_save_deposit() {
        let mut gui = initialize_gui(None).await;

        gui.save_deposit("token1".to_string(), "999".to_string())
            .unwrap();

        let deposit = gui.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);
        assert_eq!(deposit[0].token, "token1");
        assert_eq!(deposit[0].amount, "999");
        assert_eq!(
            deposit[0].address,
            Address::from_str("0xc2132d05d31c914a87c6611c10748aeb04b58e8f").unwrap()
        );

        gui.save_deposit("token1".to_string(), "".to_string())
            .unwrap();
        let deposit = gui.get_deposits().unwrap();
        assert_eq!(deposit.len(), 0);

        let mut gui = initialize_gui_with_select_tokens().await;
        let err = gui
            .save_deposit("token3".to_string(), "999".to_string())
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositTokenNotFound("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The deposit token 'token3' was not found in the YAML configuration."
        );
    }

    #[wasm_bindgen_test]
    async fn test_remove_deposit() {
        let mut gui = initialize_gui(None).await;

        gui.save_deposit("token1".to_string(), "999".to_string())
            .unwrap();
        let deposit = gui.get_deposits().unwrap();
        assert_eq!(deposit.len(), 1);

        gui.remove_deposit("token1".to_string()).unwrap();
        assert_eq!(gui.get_deposits().unwrap().len(), 0);
    }

    #[wasm_bindgen_test]
    async fn test_get_deposit_presets() {
        let gui = initialize_gui(None).await;

        let presets = gui.get_deposit_presets("token1".to_string()).unwrap();
        assert_eq!(
            presets,
            vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ]
        );
    }
    #[wasm_bindgen_test]
    async fn test_get_missing_deposits() {
        let gui = initialize_gui(None).await;

        let missing_deposits = gui.get_missing_deposits().unwrap();
        assert_eq!(missing_deposits, vec!["token1".to_string()]);
    }

    #[wasm_bindgen_test]
    async fn test_has_any_deposit() {
        let mut gui = initialize_gui(None).await;

        let has_any_deposit = gui.has_any_deposit().unwrap();
        assert!(!has_any_deposit);

        gui.save_deposit("token1".to_string(), "999".to_string())
            .unwrap();
        let has_any_deposit = gui.has_any_deposit().unwrap();
        assert!(has_any_deposit);
    }

    #[wasm_bindgen_test]
    async fn test_check_deposits() {
        let mut gui = initialize_gui(None).await;

        gui.save_deposit("token1".to_string(), "999".to_string())
            .unwrap();
        gui.check_deposits().unwrap();
        gui.remove_deposit("token1".to_string()).unwrap();

        let err = gui.check_deposits().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::DepositNotSet("T1".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "A deposit for token 'T1' is required but has not been set."
        );

        let gui = initialize_gui_with_select_tokens().await;
        let err = gui.check_deposits().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::MissingDepositToken("select-token-deployment".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "A deposit for token is required but has not been set for deployment 'select-token-deployment'."
        );
    }
}
