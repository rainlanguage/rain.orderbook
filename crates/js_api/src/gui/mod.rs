use alloy::primitives::Address;
use rain_orderbook_app_settings::gui::{
    Gui, GuiDeployment, GuiFieldDefinition, ParseGuiConfigSourceError,
};
use rain_orderbook_bindings::impl_wasm_traits;
use rain_orderbook_common::dotrain_order::{DotrainOrder, DotrainOrderError};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::HashMap;
use thiserror::Error;
use tsify::Tsify;
use wasm_bindgen::{
    convert::{
        js_value_vector_from_abi, js_value_vector_into_abi, FromWasmAbi, IntoWasmAbi,
        LongRefFromWasmAbi, RefFromWasmAbi, TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
    },
    describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR},
    prelude::*,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TokenDeposit {
    token: String,
    amount: String,
    #[tsify(type = "string")]
    address: Address,
}
impl_wasm_traits!(TokenDeposit);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FieldValuePair {
    binding: String,
    value: String,
}
impl_wasm_traits!(FieldValuePair);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct DotrainOrderGui {
    dotrain_order: DotrainOrder,
    deployment: GuiDeployment,
    field_values: HashMap<String, String>,
    deposits: Vec<TokenDeposit>,
}
#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "init")]
    pub async fn init(
        dotrain: String,
        deployment_name: String,
    ) -> Result<DotrainOrderGui, GuiError> {
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;

        let config = dotrain_order.config();
        if config.gui.is_none() {
            return Err(GuiError::GuiConfigNotFound);
        }

        let gui_config = config.gui.clone().unwrap();

        let gui_deployment = gui_config
            .deployments
            .iter()
            .find(|deployment| deployment.deployment_name == deployment_name)
            .ok_or(GuiError::DeploymentNotFound(deployment_name))?;

        Ok(Self {
            dotrain_order,
            deployment: gui_deployment.clone(),
            field_values: HashMap::new(),
            deposits: vec![],
        })
    }

    #[wasm_bindgen(js_name = "getGuiConfig")]
    pub fn get_gui_config(&self) -> Gui {
        self.dotrain_order.config().gui.clone().unwrap()
    }

    #[wasm_bindgen(js_name = "getDeposits")]
    pub fn get_deposits(&self) -> Vec<TokenDeposit> {
        self.deposits.clone()
    }

    #[wasm_bindgen(js_name = "saveDeposit")]
    pub fn save_deposit(&mut self, deposit: TokenDeposit) -> Result<(), GuiError> {
        if !self.deposits.iter().any(|d| d.token == deposit.token) {
            self.deposits.push(deposit);
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeDeposit")]
    pub fn remove_deposit(&mut self, token: String) {
        self.deposits.retain(|deposit| deposit.token != token);
    }

    #[wasm_bindgen(js_name = "saveFieldValue")]
    pub fn save_field_value(&mut self, binding: String, value: String) -> Result<(), GuiError> {
        self.deployment
            .fields
            .iter()
            .find(|field| field.binding == binding)
            .ok_or(GuiError::FieldNotFound(binding.clone()))?;
        self.field_values.insert(binding, value);
        Ok(())
    }

    #[wasm_bindgen(js_name = "saveFieldValues")]
    pub fn save_field_values(&mut self, field_values: Vec<FieldValuePair>) -> Result<(), GuiError> {
        for field_value in field_values {
            self.save_field_value(field_value.binding, field_value.value)?;
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = "getFieldValue")]
    pub fn get_field_value(&self, binding: String) -> Result<String, GuiError> {
        let field_value = self
            .field_values
            .get(&binding)
            .ok_or(GuiError::FieldNotFound(binding))?;
        Ok(field_value.clone())
    }

    #[wasm_bindgen(js_name = "getAllFieldValues")]
    pub fn get_all_field_values(&self) -> Vec<FieldValuePair> {
        self.field_values
            .iter()
            .map(|(k, v)| FieldValuePair {
                binding: k.clone(),
                value: v.clone(),
            })
            .collect()
    }

    #[wasm_bindgen(js_name = "getFieldDefinition")]
    pub fn get_field_definition(&self, binding: String) -> Result<GuiFieldDefinition, GuiError> {
        let field_definition = self
            .deployment
            .fields
            .iter()
            .find(|field| field.binding == binding)
            .ok_or(GuiError::FieldNotFound(binding))?;
        Ok(field_definition.clone())
    }

    #[wasm_bindgen(js_name = "getAllFieldDefinitions")]
    pub fn get_all_field_definitions(&self) -> Vec<GuiFieldDefinition> {
        self.deployment.fields.clone()
    }
}

#[derive(Error, Debug)]
pub enum GuiError {
    #[error("Gui config not found")]
    GuiConfigNotFound,
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),
    #[error(transparent)]
    ParseGuiConfigSourceError(#[from] ParseGuiConfigSourceError),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}
impl From<GuiError> for JsValue {
    fn from(value: GuiError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
