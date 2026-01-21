use super::*;
use rain_orderbook_app_settings::{
    gui::GuiDepositCfg,
    order::{OrderIOCfg, VaultType},
    token::TokenCfg,
};
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
    vault_ids: BTreeMap<(VaultType, String), Option<String>>,
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
    ) -> Result<BTreeMap<(VaultType, String), Option<String>>, GuiError> {
        let mut vault_ids = BTreeMap::new();
        let r#type = if is_input {
            VaultType::Input
        } else {
            VaultType::Output
        };
        for (token, vault_id) in OrderCfg::parse_vault_ids(documents, order_key, r#type)? {
            vault_ids.insert((r#type, token), vault_id.as_ref().map(|v| v.to_string()));
        }
        Ok(vault_ids)
    }

    /// Exports the complete GUI state as a compressed, encoded string.
    ///
    /// Serializes all current configuration including field values, deposits,
    /// selected tokens, and vault IDs into a compact format for persistence
    /// or sharing. The output is gzipped and base64-encoded.
    ///
    /// ## State Contents
    ///
    /// - Field values with preset information
    /// - Deposit amounts with preset references
    /// - Selected token configurations
    /// - Vault ID assignments
    /// - Configuration hash for validation
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.serializeState();
    /// if (result.error) {
    ///   console.error("Serialization error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const state = result.value;
    /// // Do something with the state
    /// ```
    #[wasm_export(
        js_name = "serializeState",
        unchecked_return_type = "string",
        return_description = "Compressed, base64-encoded state data"
    )]
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

    /// Restores a GUI instance from previously serialized state.
    ///
    /// Creates a new GUI instance with all configuration restored from a saved state.
    /// The dotrain content must match the original for security validation.
    ///
    /// ## Security
    ///
    /// The function validates that the dotrain content hasn't changed by comparing
    /// hashes. This prevents state injection attacks and ensures consistency.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await DotrainOrderGui.newFromState(dotrainYaml, savedState);
    /// if (result.error) {
    ///   console.error("Restore failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const gui = result.value;
    /// // Do something with the gui
    /// ```
    #[wasm_export(
        js_name = "newFromState",
        preserve_js_class,
        return_description = "Fully restored GUI instance"
    )]
    pub async fn new_from_state(
        #[wasm_export(param_description = "Must match the original dotrain content exactly")]
        dotrain: String,
        #[wasm_export(param_description = "Previously serialized state string")] serialized: String,
        #[wasm_export(param_description = "Optional callback for future state changes")]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<DotrainOrderGui, GuiError> {
        let compressed = URL_SAFE.decode(serialized)?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;

        let original_dotrain_hash = DotrainOrderGui::get_dotrain_hash(dotrain.clone())?;
        let state: SerializedGuiState = bincode::deserialize(&bytes)?;

        if original_dotrain_hash != state.dotrain_hash {
            return Err(GuiError::DotrainMismatch);
        }
        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await?;

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
                .get_order_for_gui_deployment(&order_key, &state.selected_deployment)
                .and_then(|mut order| order.update_vault_id(is_input, index, vault_id))?;
        }

        Ok(dotrain_order_gui)
    }

    /// Manually triggers the state update callback.
    ///
    /// Calls the registered state update callback with the current serialized state.
    /// This is typically called automatically after state-changing operations,
    /// but can be triggered manually if needed.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Manual state update trigger
    /// const result = gui.executeStateUpdateCallback();
    /// if (result.error) {
    ///   console.error("Callback error:", result.error.readableMsg);
    /// }
    /// ```
    #[wasm_export(
        js_name = "executeStateUpdateCallback",
        unchecked_return_type = "void",
        return_description = "Callback executed successfully or no callback registered"
    )]
    pub fn execute_state_update_callback(&self) -> Result<(), GuiError> {
        if let Some(callback) = &self.state_update_callback {
            let state = to_js_value(&self.serialize_state()?)?;
            callback.call1(&JsValue::UNDEFINED, &state).map_err(|e| {
                GuiError::JsError(format!("Failed to execute state update callback: {:?}", e))
            })?;
        }
        Ok(())
    }

    /// Gets comprehensive configuration for complete UI initialization.
    ///
    /// Provides all configuration data needed to build a complete order interface,
    /// organized by requirement type for progressive UI construction.
    ///
    /// ## Configuration Structure
    ///
    /// - `fieldDefinitionsWithoutDefaults` - Required fields needing user input
    /// - `fieldDefinitionsWithDefaults` - Optional fields with fallback values
    /// - `deposits` - Deposit configurations with tokens and presets
    /// - `orderInputs` - Input token configurations
    /// - `orderOutputs` - Output token configurations
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getAllGuiConfig();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const config = result.value;
    ///
    /// // Build required fields section
    /// config.fieldDefinitionsWithoutDefaults.forEach(field => {
    ///   // Do something with the fields
    /// });
    ///
    /// // Build optional fields section
    /// if (config.fieldDefinitionsWithDefaults.length > 0) {
    ///   // Do something with the fields
    /// }
    ///
    /// // Build deposits section
    /// config.deposits.forEach(deposit => {
    ///   // Do something with the deposits
    /// });
    ///
    /// // Show order preview
    /// config.orderInputs.forEach(input => {
    ///   // Do something with the order inputs
    /// });
    /// config.orderOutputs.forEach(output => {
    ///   // Do something with the order outputs
    /// });
    /// ```
    #[wasm_export(
        js_name = "getAllGuiConfig",
        unchecked_return_type = "AllGuiConfig",
        return_description = "Complete configuration package"
    )]
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
        tests::{get_yaml, initialize_gui_with_select_tokens},
    };
    use alloy::primitives::U256;
    use js_sys::{eval, Reflect};
    use rain_orderbook_app_settings::order::VaultType;
    use wasm_bindgen_test::wasm_bindgen_test;

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
        gui.set_deposit("token3".to_string(), "100".to_string())
            .await
            .unwrap();
        gui.set_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();
        gui.set_vault_id(
            VaultType::Input,
            "token1".to_string(),
            Some("199".to_string()),
        )
        .unwrap();
        gui.set_vault_id(
            VaultType::Output,
            "token2".to_string(),
            Some("299".to_string()),
        )
        .unwrap();

        let state = gui.serialize_state().unwrap();
        assert!(!state.is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_new_from_state() {
        let mut gui = initialize_gui_with_select_tokens().await;

        gui.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "TKN3".to_string(),
        );
        gui.set_deposit("token3".to_string(), "100".to_string())
            .await
            .unwrap();
        gui.set_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();
        gui.set_vault_id(
            VaultType::Input,
            "token1".to_string(),
            Some("199".to_string()),
        )
        .unwrap();
        gui.set_vault_id(
            VaultType::Output,
            "token2".to_string(),
            Some("299".to_string()),
        )
        .unwrap();

        let serialized_state = gui.serialize_state().unwrap();

        let gui = DotrainOrderGui::new_from_state(get_yaml(), serialized_state.to_string(), None)
            .await
            .unwrap();

        assert!(gui.is_select_token_set("token3".to_string()).unwrap());
        assert_eq!(gui.get_deposits().unwrap()[0].amount, "100");
        assert_eq!(
            gui.get_field_value("binding-1".to_string()).unwrap(),
            FieldValue {
                field: "binding-1".to_string(),
                value: "100".to_string(),
                is_preset: false,
            }
        );
        assert_eq!(
            gui.get_field_value("binding-2".to_string()).unwrap(),
            FieldValue {
                field: "binding-2".to_string(),
                value: "0".to_string(),
                is_preset: true,
            }
        );
        let vault_ids = gui.get_vault_ids().unwrap().0;
        assert_eq!(
            vault_ids.get("input").unwrap()["token1"],
            Some(U256::from(199))
        );
        assert_eq!(
            vault_ids.get("output").unwrap()["token2"],
            Some(U256::from(299))
        );
    }

    #[wasm_bindgen_test]
    async fn test_new_from_state_invalid_dotrain() {
        let gui = initialize_gui_with_select_tokens().await;
        let serialized_state = gui.serialize_state().unwrap();

        let different_dotrain = r#"
        dotrain:
            name: Test
            description: Test
        "#;

        let err =
            DotrainOrderGui::new_from_state(different_dotrain.to_string(), serialized_state, None)
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

        let mut gui = DotrainOrderGui::new_with_deployment(
            get_yaml(),
            "some-deployment".to_string(),
            Some(callback_js.clone()),
        )
        .await
        .unwrap();

        let callback_called = Reflect::get(&global, &JsValue::from_str("callbackCalled"))
            .expect("should have callbackCalled flag on globalThis");
        assert_eq!(callback_called, JsValue::from_bool(false));

        gui.set_deposit("token1".to_string(), "100".to_string())
            .await
            .unwrap();
        gui.set_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "582.1".to_string())
            .unwrap();
        gui.set_vault_id(
            VaultType::Input,
            "token1".to_string(),
            Some("199".to_string()),
        )
        .unwrap();
        gui.set_vault_id(
            VaultType::Output,
            "token2".to_string(),
            Some("299".to_string()),
        )
        .unwrap();

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
