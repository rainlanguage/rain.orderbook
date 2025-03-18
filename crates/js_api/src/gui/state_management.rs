use super::*;
use rain_orderbook_app_settings::{gui::GuiDepositCfg, order::OrderIOCfg, token::TokenCfg};
use sha2::{Digest, Sha256};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]

pub struct YamlFields {
    pub field_definitions_without_defaults: Vec<GuiFieldDefinitionCfg>,
    pub field_definitions_with_defaults: Vec<GuiFieldDefinitionCfg>,
    pub deposits: Vec<GuiDepositCfg>,
    pub order_inputs: Vec<OrderIOCfg>,
    pub order_outputs: Vec<OrderIOCfg>,
}
impl_wasm_traits!(YamlFields);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    field_values: BTreeMap<String, GuiPresetCfg>,
    deposits: BTreeMap<String, GuiPresetCfg>,
    select_tokens: BTreeMap<String, TokenCfg>,
    vault_ids: BTreeMap<(bool, u8), Option<String>>,
    dotrain_hash: String,
    selected_deployment: String,
}

#[wasm_bindgen]
impl DotrainOrderGui {
    fn get_dotrain_hash(dotrain: &str) -> Result<String, GuiError> {
        let dotrain_bytes = bincode::serialize(dotrain)?;
        let hash = Sha256::digest(&dotrain_bytes);
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
        if preset.id != "" {
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

    #[wasm_bindgen(js_name = "serializeState")]
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
            dotrain_hash: DotrainOrderGui::get_dotrain_hash(&self.dotrain_order.dotrain())?,
            selected_deployment: self.selected_deployment.clone(),
        };
        let bytes = bincode::serialize(&state)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let compressed = encoder.finish()?;

        Ok(URL_SAFE.encode(compressed))
    }

    #[wasm_bindgen(js_name = "deserializeState")]
    pub async fn deserialize_state(
        dotrain: String,
        serialized: String,
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<DotrainOrderGui, GuiError> {
        let compressed = URL_SAFE.decode(serialized)?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;

        let original_dotrain_hash = DotrainOrderGui::get_dotrain_hash(&dotrain)?;
        let state: SerializedGuiState = bincode::deserialize(&bytes)?;

        if original_dotrain_hash != state.dotrain_hash {
            return Err(GuiError::DotrainMismatch);
        }
        let dotrain_order = DotrainOrder::new(dotrain, None).await?;

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

        Ok(dotrain_order_gui)
    }

    #[wasm_bindgen(js_name = "clearState")]
    pub fn clear_state(&mut self) {
        self.field_values.clear();
        self.deposits.clear();
    }

    fn is_preset<K: AsRef<str>>(
        &self,
        key: K,
        map: &BTreeMap<String, field_values::PairValue>,
    ) -> Option<bool> {
        map.get(key.as_ref()).map(|v| v.is_preset)
    }

    #[wasm_bindgen(js_name = "isFieldPreset")]
    pub fn is_field_preset(&self, binding: String) -> Option<bool> {
        self.is_preset(binding, &self.field_values)
    }

    #[wasm_bindgen(js_name = "isDepositPreset")]
    pub fn is_deposit_preset(&self, token: String) -> Option<bool> {
        self.is_preset(token, &self.deposits)
    }

    #[wasm_bindgen(js_name = "executeStateUpdateCallback")]
    pub fn execute_state_update_callback(&self) -> Result<(), GuiError> {
        if let Some(callback) = &self.state_update_callback {
            let state = to_js_value(&self.serialize_state()?)?;
            callback.call1(&JsValue::UNDEFINED, &state).map_err(|e| {
                GuiError::JsError(format!("Failed to execute state update callback: {:?}", e))
            })?;
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = "getYamlFields")]
    pub fn get_yaml_fields(&self) -> Result<YamlFields, GuiError> {
        let deployment = self.get_current_deployment()?;

        let field_definitions_without_defaults = self.get_all_field_definitions(Some(false))?;
        let field_definitions_with_defaults = self.get_all_field_definitions(Some(true))?;
        let deposits = deployment.deposits;
        let order_inputs = deployment.deployment.order.inputs.clone();
        let order_outputs = deployment.deployment.order.outputs.clone();

        let yaml_fields = YamlFields {
            field_definitions_without_defaults,
            field_definitions_with_defaults,
            deposits,
            order_inputs,
            order_outputs,
        };
        Ok(yaml_fields)
    }
}
