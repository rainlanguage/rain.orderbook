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
        let current_deployment = self.get_current_deployment()?;
        self.deposits
            .iter()
            .map(|(token, value)| {
                let gui_deposit = current_deployment
                    .deposits
                    .iter()
                    .find(|dg| dg.token.key == *token)
                    .ok_or(GuiError::DepositTokenNotFound(token.clone()))?;
                let amount: String = if value.is_preset {
                    gui_deposit
                        .presets
                        .get(value.value.parse::<usize>().unwrap())
                        .ok_or(GuiError::InvalidPreset)?
                        .clone()
                } else {
                    value.value.clone()
                };
                Ok(TokenDeposit {
                    token: gui_deposit.token.key.clone(),
                    amount,
                    address: gui_deposit.token.address,
                })
            })
            .collect::<Result<Vec<TokenDeposit>, GuiError>>()
    }

    #[wasm_bindgen(js_name = "saveDeposit")]
    pub fn save_deposit(&mut self, token: String, amount: String) -> Result<(), GuiError> {
        let current_deployment = self.get_current_deployment()?;
        let gui_deposit = current_deployment
            .deposits
            .iter()
            .find(|dg| dg.token.key == token)
            .ok_or(GuiError::DepositTokenNotFound(token.clone()))?;

        let value = if let Some(index) = gui_deposit.presets.iter().position(|p| **p == amount) {
            field_values::PairValue {
                is_preset: true,
                value: index.to_string(),
            }
        } else {
            field_values::PairValue {
                is_preset: false,
                value: amount,
            }
        };

        self.deposits.insert(token, value);
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeDeposit")]
    pub fn remove_deposit(&mut self, token: String) {
        self.deposits.remove(&token);
    }

    #[wasm_bindgen(js_name = "getDepositPresets")]
    pub fn get_deposit_presets(&self, token: String) -> Result<Vec<String>, GuiError> {
        let current_deployment = self.get_current_deployment()?;
        let gui_deposit = current_deployment
            .deposits
            .iter()
            .find(|dg| dg.token.key == token)
            .ok_or(GuiError::DepositTokenNotFound(token.clone()))?;
        Ok(gui_deposit.presets.clone())
    }
}
