use super::*;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    field_values: BTreeMap<String, GuiPreset>,
    deposits: BTreeMap<String, GuiPreset>,
    select_tokens: Option<BTreeMap<String, String>>,
    vault_ids: BTreeMap<(bool, u8), Option<String>>,
}

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "serializeState")]
    pub fn serialize(&self) -> Result<String, GuiError> {
        let deployment = self.get_current_deployment()?;

        let mut field_values = BTreeMap::new();
        for (k, v) in self.field_values.iter() {
            let preset = if v.is_preset {
                let field_definition = self.get_field_definition(k)?;
                let presets = field_definition
                    .presets
                    .ok_or(GuiError::BindingHasNoPresets(k.clone()))?;
                presets
                    .iter()
                    .find(|preset| preset.id == v.value)
                    .ok_or(GuiError::InvalidPreset)?
                    .clone()
            } else {
                GuiPreset {
                    id: "".to_string(),
                    name: None,
                    value: v.value.clone(),
                }
            };
            field_values.insert(k.clone(), preset);
        }

        let mut deposits = BTreeMap::new();
        for (k, v) in self.deposits.iter() {
            let preset = if v.is_preset {
                GuiPreset {
                    id: v.value.clone(),
                    name: None,
                    value: String::default(),
                }
            } else {
                GuiPreset {
                    id: "".to_string(),
                    name: None,
                    value: v.value.clone(),
                }
            };
            deposits.insert(k.clone(), preset);
        }

        let mut select_tokens: Option<BTreeMap<String, String>> = None;
        if let Some(tokens) = &self.select_tokens {
            select_tokens = Some(
                tokens
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k, v.to_string()))
                    .collect(),
            );
        }

        let mut vault_ids = BTreeMap::new();
        for (i, input) in deployment.deployment.order.inputs.iter().enumerate() {
            vault_ids.insert((true, i as u8), input.vault_id.map(|v| v.to_string()));
        }
        for (i, output) in deployment.deployment.order.outputs.iter().enumerate() {
            vault_ids.insert((false, i as u8), output.vault_id.map(|v| v.to_string()));
        }

        let state = SerializedGuiState {
            field_values: field_values.clone(),
            deposits: deposits.clone(),
            select_tokens,
            vault_ids,
        };
        let bytes = bincode::serialize(&state)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let compressed = encoder.finish()?;

        Ok(URL_SAFE.encode(compressed))
    }

    #[wasm_bindgen(js_name = "deserializeState")]
    pub fn deserialize_state(&mut self, serialized: String) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        let compressed = URL_SAFE.decode(serialized)?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;

        let state: SerializedGuiState = bincode::deserialize(&bytes)?;

        let field_values = state
            .field_values
            .into_iter()
            .map(|(k, v)| {
                let pair_value = if v.id != "" {
                    field_values::PairValue {
                        is_preset: true,
                        value: v.id,
                    }
                } else {
                    field_values::PairValue {
                        is_preset: false,
                        value: v.value,
                    }
                };
                (k, pair_value)
            })
            .collect::<BTreeMap<_, _>>();

        let deposits = state
            .deposits
            .into_iter()
            .map(|(k, v)| {
                let pair_value = if v.id != "" {
                    field_values::PairValue {
                        is_preset: true,
                        value: v.id,
                    }
                } else {
                    field_values::PairValue {
                        is_preset: false,
                        value: v.value,
                    }
                };
                (k, pair_value)
            })
            .collect::<BTreeMap<_, _>>();

        let select_tokens = if let Some(tokens) = state.select_tokens {
            let mut result = BTreeMap::new();
            for (k, v) in tokens {
                result.insert(k, Address::from_str(&v)?);
            }
            Some(result)
        } else {
            None
        };

        self.field_values = field_values;
        self.deposits = deposits;
        self.select_tokens = select_tokens;
        for ((is_input, index), vault_id) in state.vault_ids {
            self.dotrain_order
                .dotrain_yaml()
                .get_order(&deployment.deployment.order.key)
                .and_then(|mut order| {
                    order.update_vault_id(is_input, index, vault_id.unwrap_or_default())
                })?;
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "clearState")]
    pub fn clear_state(&mut self) {
        self.field_values.clear();
        self.deposits.clear();
        self.select_tokens = None;
    }

    #[wasm_bindgen(js_name = "isFieldPreset")]
    pub fn is_field_preset(&self, binding: String) -> Option<bool> {
        let value = self.field_values.get(&binding);
        value.map(|v| v.is_preset)
    }

    #[wasm_bindgen(js_name = "isDepositPreset")]
    pub fn is_deposit_preset(&self, token: String) -> Option<bool> {
        let value = self.deposits.get(&token);
        value.map(|v| v.is_preset)
    }
}
