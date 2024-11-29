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
    pub fn get_deposits(&self) -> Vec<TokenDeposit> {
        self.deposits.clone()
    }

    #[wasm_bindgen(js_name = "saveDeposit")]
    pub fn save_deposit(&mut self, token: String, amount: String) -> Result<(), GuiError> {
        let gui_deposit = self
            .deployment
            .deposits
            .iter()
            .find(|dg| dg.token_name == token)
            .ok_or(GuiError::DepositTokenNotFound(token.clone()))?;

        let deposit_token = TokenDeposit {
            token: gui_deposit.token_name.clone(),
            amount,
            address: gui_deposit.token.address,
        };

        if !self.deposits.iter().any(|d| d.token == token) {
            self.deposits.push(deposit_token);
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeDeposit")]
    pub fn remove_deposit(&mut self, token: String) {
        self.deposits.retain(|deposit| deposit.token != token);
    }
}
