use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TokenDeposit {
    pub token: String,
    pub amount: String,
    #[tsify(type = "string")]
    pub address: Address,
}
impl_all_wasm_traits!(TokenDeposit);

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "getDeposits")]
    pub fn get_deposits(&self) -> Result<Vec<TokenDeposit>, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.deposits
            .iter()
            .map(|(key, value)| {
                let gui_deposit = deployment
                    .deposits
                    .iter()
                    .find(|dg| dg.token.as_ref().map_or(false, |t| t.key == *key))
                    .ok_or(GuiError::DepositTokenNotFound(key.clone()))?;
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

    #[wasm_bindgen(js_name = "saveDeposit")]
    pub fn save_deposit(&mut self, token: String, amount: String) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        let gui_deposit = deployment
            .deposits
            .iter()
            .find(|dg| dg.token.as_ref().map_or(false, |t| t.key == token))
            .ok_or(GuiError::DepositTokenNotFound(token.clone()))?;

        if amount.is_empty() {
            self.remove_deposit(token);
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
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeDeposit")]
    pub fn remove_deposit(&mut self, token: String) {
        self.deposits.remove(&token);
    }

    #[wasm_bindgen(js_name = "getDepositPresets")]
    pub fn get_deposit_presets(&self, key: String) -> Result<Vec<String>, GuiError> {
        let deployment = self.get_current_deployment()?;
        let gui_deposit = deployment
            .deposits
            .iter()
            .find(|dg| dg.token.as_ref().map_or(false, |t| t.key == key))
            .ok_or(GuiError::DepositTokenNotFound(key.clone()))?;
        Ok(gui_deposit.presets.clone().unwrap_or(vec![]))
    }

    pub fn check_deposits(&self) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;

        for deposit in deployment.deposits.iter() {
            if deposit.token.is_none() {
                return Err(GuiError::TokenMustBeSelected("deposit".to_string()));
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

    #[wasm_bindgen(js_name = "getMissingDeposits")]
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
}
