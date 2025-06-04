use super::*;
use rain_orderbook_app_settings::{gui::GuiDepositCfg, order::OrderIOCfg, token::TokenCfg};
use sha2::{Digest, Sha256};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AllGuiConfig {
    pub field_definitions_without_defaults: Vec<GuiFieldDefinitionCfg>,
    pub field_definitions_with_defaults: Vec<GuiFieldDefinitionCfg>,
    pub deposits: Vec<GuiDepositCfg>,
    pub order_inputs: Vec<OrderIOCfg>,
    pub order_outputs: Vec<OrderIOCfg>,
}
impl_wasm_traits!(AllGuiConfig);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    field_values: BTreeMap<String, GuiPresetCfg>,
    deposits: BTreeMap<String, GuiPresetCfg>,
    select_tokens: BTreeMap<String, TokenCfg>,
    vault_ids: BTreeMap<(bool, u8), Option<String>>,
    dotrain_hash: String,
    selected_deployment: String,
}

#[wasm_export]
impl DotrainOrderGui {
    fn get_dotrain_hash(dotrain: String) -> Result<String, GuiError> {
        let dotrain_bytes = bincode::serialize(&dotrain)?;
        let hash = Sha256::digest(dotrain_bytes);
        Ok(URL_SAFE.encode(hash))
    }

    fn create_preset(value: &field_values::PairValue, default_value: String) -> GuiPresetCfg {
        if value.is_preset {
            GuiPresetCfg {
                id: value.value.clone(),
                name: None,
                value: default_value,
            }
        } else {
            GuiPresetCfg {
                id: "".to_string(),
                name: None,
                value: value.value.clone(),
            }
        }
    }

    fn preset_to_pair_value(preset: GuiPresetCfg) -> field_values::PairValue {
        if !preset.id.is_empty() {
            field_values::PairValue {
                is_preset: true,
                value: preset.id,
            }
        } else {
            field_values::PairValue {
                is_preset: false,
                value: preset.value,
            }
        }
    }

    fn parse_vault_ids_for_order(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        order_key: &str,
        is_input: bool,
    ) -> Result<BTreeMap<(bool, u8), Option<String>>, GuiError> {
        let mut vault_ids = BTreeMap::new();
        for (i, vault_id) in OrderCfg::parse_vault_ids(documents, order_key, is_input)?
            .iter()
            .enumerate()
        {
            vault_ids.insert(
                (is_input, i as u8),
                vault_id.as_ref().map(|v| v.to_string()),
            );
        }
        Ok(vault_ids)
    }

    #[wasm_export(js_name = "serializeState", unchecked_return_type = "string")]
    pub fn serialize_state(&self) -> Result<String, GuiError> {
        let mut field_values = BTreeMap::new();
        for (k, v) in self.field_values.iter() {
            let preset = if v.is_preset {
                let presets = GuiCfg::parse_field_presets(
                    self.dotrain_order.dotrain_yaml().documents.clone(),
                    &self.selected_deployment,
                    k,
                )?
                .ok_or(GuiError::BindingHasNoPresets(k.clone()))?;
                presets
                    .iter()
                    .find(|preset| preset.id == v.value)
                    .ok_or(GuiError::InvalidPreset)?
                    .clone()
            } else {
                Self::create_preset(v, String::default())
            };
            field_values.insert(k.clone(), preset);
        }

        let mut deposits = BTreeMap::new();
        for (k, v) in self.deposits.iter() {
            let preset = Self::create_preset(v, String::default());
            deposits.insert(k.clone(), preset);
        }

        let mut select_tokens: BTreeMap<String, TokenCfg> = BTreeMap::new();
        if let Some(st) = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &self.selected_deployment,
        )? {
            for select_token in st {
                if let Ok(token) = self
                    .dotrain_order
                    .orderbook_yaml()
                    .get_token(&select_token.key)
                {
                    select_tokens.insert(select_token.key, token);
                }
            }
        }

        let order_key = DeploymentCfg::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &self.selected_deployment,
        )?;
        let mut vault_ids = BTreeMap::new();
        vault_ids.extend(Self::parse_vault_ids_for_order(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &order_key,
            true,
        )?);
        vault_ids.extend(Self::parse_vault_ids_for_order(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &order_key,
            false,
        )?);

        let state = SerializedGuiState {
            field_values: field_values.clone(),
            deposits: deposits.clone(),
            select_tokens: select_tokens.clone(),
            vault_ids: vault_ids.clone(),
            dotrain_hash: DotrainOrderGui::get_dotrain_hash(self.dotrain_order.dotrain()?)?,
            selected_deployment: self.selected_deployment.clone(),
        };
        let bytes = bincode::serialize(&state)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let compressed = encoder.finish()?;

        Ok(URL_SAFE.encode(compressed))
    }

    #[wasm_export(js_name = "deserializeState", unchecked_return_type = "void")]
    pub async fn deserialize_state(
        &mut self,
        dotrain: String,
        serialized: String,
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<(), GuiError> {
        let compressed = URL_SAFE.decode(serialized)?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;

        let original_dotrain_hash = DotrainOrderGui::get_dotrain_hash(dotrain.clone())?;
        let state: SerializedGuiState = bincode::deserialize(&bytes)?;

        if original_dotrain_hash != state.dotrain_hash {
            return Err(GuiError::DotrainMismatch);
        }
        let mut dotrain_order = DotrainOrder::new();
        dotrain_order.initialize(dotrain.clone(), None).await?;

        let field_values = state
            .field_values
            .into_iter()
            .map(|(k, v)| (k, Self::preset_to_pair_value(v)))
            .collect::<BTreeMap<_, _>>();

        let deposits = state
            .deposits
            .into_iter()
            .map(|(k, v)| (k, Self::preset_to_pair_value(v)))
            .collect::<BTreeMap<_, _>>();

        let dotrain_order_gui = DotrainOrderGui {
            dotrain_order,
            field_values,
            deposits,
            selected_deployment: state.selected_deployment.clone(),
            state_update_callback,
        };

        let deployment_select_tokens = GuiCfg::parse_select_tokens(
            dotrain_order_gui.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for (key, token) in state.select_tokens {
            let select_tokens = deployment_select_tokens
                .as_ref()
                .ok_or(GuiError::SelectTokensNotSet)?;
            if !select_tokens.iter().any(|token| token.key == key) {
                return Err(GuiError::TokenNotInSelectTokens(key));
            }
            if dotrain_order_gui.is_select_token_set(key.clone())? {
                TokenCfg::remove_record_from_yaml(
                    dotrain_order_gui.dotrain_order.orderbook_yaml().documents,
                    &key,
                )?;
            }
            TokenCfg::add_record_to_yaml(
                dotrain_order_gui.dotrain_order.orderbook_yaml().documents,
                &key,
                &token.network.key,
                &token.address.to_string(),
                token.decimals.map(|d| d.to_string()).as_deref(),
                token.label.map(|l| l.to_string()).as_deref(),
                token.symbol.map(|s| s.to_string()).as_deref(),
            )?;
        }

        let order_key = DeploymentCfg::parse_order_key(
            dotrain_order_gui.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for ((is_input, index), vault_id) in state.vault_ids {
            dotrain_order_gui
                .dotrain_order
                .dotrain_yaml()
                .get_order(&order_key)
                .and_then(|mut order| order.update_vault_id(is_input, index, vault_id))?;
        }

        self.dotrain_order = dotrain_order_gui.dotrain_order;
        self.field_values = dotrain_order_gui.field_values;
        self.deposits = dotrain_order_gui.deposits;
        self.selected_deployment = dotrain_order_gui.selected_deployment;
        self.state_update_callback = dotrain_order_gui.state_update_callback;

        Ok(())
    }

    #[wasm_export(js_name = "executeStateUpdateCallback", unchecked_return_type = "void")]
    pub fn execute_state_update_callback(&self) -> Result<(), GuiError> {
        if let Some(callback) = &self.state_update_callback {
            let state = to_js_value(&self.serialize_state()?)?;
            callback.call1(&JsValue::UNDEFINED, &state).map_err(|e| {
                GuiError::JsError(format!("Failed to execute state update callback: {:?}", e))
            })?;
        }
        Ok(())
    }

    #[wasm_export(js_name = "getAllGuiConfig", unchecked_return_type = "AllGuiConfig")]
    pub fn get_all_gui_config(&self) -> Result<AllGuiConfig, GuiError> {
        let deployment = self.get_current_deployment()?;

        let field_definitions_without_defaults = self.get_all_field_definitions(Some(false))?;
        let field_definitions_with_defaults = self.get_all_field_definitions(Some(true))?;
        let deposits = deployment.deposits;
        let order_inputs = deployment.deployment.order.inputs.clone();
        let order_outputs = deployment.deployment.order.outputs.clone();

        Ok(AllGuiConfig {
            field_definitions_without_defaults,
            field_definitions_with_defaults,
            deposits,
            order_inputs,
            order_outputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        field_values::FieldValue,
        tests::{get_yaml, initialize_gui, initialize_gui_with_select_tokens},
    };
    use alloy::primitives::U256;
    use js_sys::{eval, Reflect};
    use wasm_bindgen_test::wasm_bindgen_test;

    const SERIALIZED_STATE: &str = "H4sIAAAAAAAA_21PyYrCQBBNO8MMA3OSgTkNzAfYpGNU0oIHwZWAGy5njY1GewlJu-FP-MmiVkcU61Dvvaqiql7GusUX4CyU81AusGOZeAN0CHkeyiMoECtlhnwAarVm0n217fXko_oGlSjBsGR6p-L1H9SWWkdl2-YqmPKlSnTZI17RjqMAb2J-NAeRYcicrg9bP0CzhfH-9JRQFn1Ce3j54d9F70b7HTeTWkkX5ilFd-VQmgPq9YUYrZK-6DabPqttB2F1SoU_jnCLTrq9dsGrDooHXFo0SOXXOGWcBRpf7eM5i7g6CCb1GedGmr-oAQAA";

    #[wasm_bindgen_test]
    async fn test_serialize_state() {
        let mut gui = initialize_gui_with_select_tokens().await;

        gui.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "TKN3".to_string(),
        );
        gui.save_deposit("token3".to_string(), "100".to_string())
            .unwrap();
        gui.save_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.save_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();
        gui.set_vault_id(true, 0, Some("199".to_string())).unwrap();
        gui.set_vault_id(false, 0, Some("299".to_string())).unwrap();

        let state = gui.serialize_state().unwrap();
        assert!(!state.is_empty());
        assert_eq!(state, SERIALIZED_STATE);
    }

    #[wasm_bindgen_test]
    async fn test_deserialize_state() {
        let mut gui = initialize_gui(None).await;
        gui.deserialize_state(get_yaml(), SERIALIZED_STATE.to_string(), None)
            .await
            .unwrap();

        assert!(gui.is_select_token_set("token3".to_string()).unwrap());
        assert_eq!(gui.get_deposits().unwrap()[0].amount, "100");
        assert_eq!(
            gui.get_field_value("binding-1".to_string()).unwrap(),
            FieldValue {
                binding: "binding-1".to_string(),
                value: "100".to_string(),
                is_preset: false,
            }
        );
        assert_eq!(
            gui.get_field_value("binding-2".to_string()).unwrap(),
            FieldValue {
                binding: "binding-2".to_string(),
                value: "0".to_string(),
                is_preset: true,
            }
        );
        let vault_ids = gui.get_vault_ids().unwrap().0;
        assert_eq!(vault_ids.get("input").unwrap()[0], Some(U256::from(199)));
        assert_eq!(vault_ids.get("output").unwrap()[0], Some(U256::from(299)));
    }

    #[wasm_bindgen_test]
    async fn test_deserialize_state_invalid_dotrain() {
        let mut gui = initialize_gui(None).await;
        let dotrain = r#"
        dotrain:
            name: Test
            description: Test
        "#;

        let err = gui
            .deserialize_state(dotrain.to_string(), SERIALIZED_STATE.to_string(), None)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), GuiError::DotrainMismatch.to_string());
        assert_eq!(
            err.to_readable_msg(),
            "There was a mismatch in the dotrain configuration. Please check your YAML configuration for consistency."
        );
    }

    #[wasm_bindgen_test]
    async fn test_execute_state_update_callback() {
        eval(
            r#"
            globalThis.callbackCalled = false;
            globalThis.receivedState = null;
            globalThis.testCallback = function(state) {
                globalThis.callbackCalled = true;
                globalThis.receivedState = state;
            };
        "#,
        )
        .unwrap();

        let global = js_sys::global();
        let callback_js = Reflect::get(&global, &JsValue::from_str("testCallback"))
            .expect("should have testCallback function on globalThis")
            .dyn_into::<js_sys::Function>()
            .expect("testCallback should be a function");

        let mut gui = DotrainOrderGui::new();
        gui.choose_deployment(
            get_yaml(),
            "some-deployment".to_string(),
            Some(callback_js.clone()),
        )
        .await
        .unwrap();

        let callback_called = Reflect::get(&global, &JsValue::from_str("callbackCalled"))
            .expect("should have callbackCalled flag on globalThis");
        assert_eq!(callback_called, JsValue::from_bool(false));

        gui.save_deposit("token1".to_string(), "100".to_string())
            .unwrap();
        gui.save_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.save_field_value("binding-2".to_string(), "582.1".to_string())
            .unwrap();
        gui.set_vault_id(true, 0, Some("199".to_string())).unwrap();
        gui.set_vault_id(false, 0, Some("299".to_string())).unwrap();

        let callback_called = Reflect::get(&global, &JsValue::from_str("callbackCalled"))
            .expect("should have callbackCalled flag on globalThis");
        assert_eq!(callback_called, JsValue::from_bool(true));

        let received_state_js = Reflect::get(&global, &JsValue::from_str("receivedState"))
            .expect("should have receivedState on globalThis");
        assert!(received_state_js.is_string());
        let received_state_rust: String = received_state_js.as_string().unwrap();
        assert!(!received_state_rust.is_empty());

        let expected_state = gui.serialize_state().unwrap();
        assert_eq!(received_state_rust, expected_state);
    }
}
