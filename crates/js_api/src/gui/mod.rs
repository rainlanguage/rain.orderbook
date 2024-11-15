use alloy::primitives::Address;
use alloy_ethers_typecast::transaction::ReadableClientError;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rain_orderbook_app_settings::gui::{
    Gui, GuiDeployment, GuiFieldDefinition, ParseGuiConfigSourceError,
};
use rain_orderbook_bindings::impl_wasm_traits;
use rain_orderbook_common::dotrain_order::{DotrainOrder, DotrainOrderError};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::BTreeMap;
use std::io::prelude::*;
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

mod order_operations;
mod state_management;

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
    field_values: BTreeMap<String, String>,
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
        let gui_config = config.gui.clone().ok_or(GuiError::GuiConfigNotFound)?;

        let gui_deployment = gui_config
            .deployments
            .iter()
            .find(|deployment| deployment.deployment_name == deployment_name)
            .ok_or(GuiError::DeploymentNotFound(deployment_name))?;

        Ok(Self {
            dotrain_order,
            deployment: gui_deployment.clone(),
            field_values: BTreeMap::new(),
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

    #[wasm_bindgen(js_name = "saveFieldValue")]
    pub fn save_field_value(&mut self, binding: String, value: String) -> Result<(), GuiError> {
        self.deployment
            .fields
            .iter()
            .find(|field| field.binding == binding)
            .ok_or(GuiError::FieldBindingNotFound(binding.clone()))?;
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
            .ok_or(GuiError::FieldBindingNotFound(binding))?;
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
            .ok_or(GuiError::FieldBindingNotFound(binding))?;
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
    #[error("Field binding not found: {0}")]
    FieldBindingNotFound(String),
    #[error("Deposit token not found in gui config: {0}")]
    DepositTokenNotFound(String),
    #[error("Orderbook not found")]
    OrderbookNotFound,
    #[error("Deserialized config mismatch")]
    DeserializedConfigMismatch,
    #[error("Vault id not found")]
    VaultIdNotFound,
    #[error("Deployer not found")]
    DeployerNotFound,
    #[error("Token not found")]
    TokenNotFound,
    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),
    #[error(transparent)]
    ParseGuiConfigSourceError(#[from] ParseGuiConfigSourceError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    FromHexError(#[from] alloy::hex::FromHexError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    DepositError(#[from] rain_orderbook_common::deposit::DepositError),
    #[error(transparent)]
    ParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error(transparent)]
    ReadContractParametersBuilderError(
        #[from] alloy_ethers_typecast::transaction::ReadContractParametersBuilderError,
    ),
    #[error(transparent)]
    UnitsError(#[from] alloy::primitives::utils::UnitsError),
    #[error(transparent)]
    WritableTransactionExecuteError(
        #[from] rain_orderbook_common::transaction::WritableTransactionExecuteError,
    ),
    #[error(transparent)]
    AddOrderArgsError(#[from] rain_orderbook_common::add_order::AddOrderArgsError),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}
impl From<GuiError> for JsValue {
    fn from(value: GuiError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
