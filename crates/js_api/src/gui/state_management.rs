use rain_orderbook_app_settings::yaml::dotrain::DotrainYaml;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    field_values: BTreeMap<String, GuiPreset>,
    deposits: BTreeMap<String, GuiPreset>,
    selected_deployment: String,
    sources: Vec<String>,
}

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "serializeState")]
    pub fn serialize_state(&self) -> Result<String, GuiError> {
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

        let sources = self.dotrain_order.get_sources()?;

        let state = SerializedGuiState {
            field_values: field_values.clone(),
            deposits: deposits.clone(),
            selected_deployment: self.selected_deployment.clone(),
            sources: sources.clone(),
        };
        let bytes = bincode::serialize(&state)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let compressed = encoder.finish()?;

        Ok(URL_SAFE.encode(compressed))
    }

    #[wasm_bindgen(js_name = "deserializeState")]
    pub async fn deserialize_state(serialized: String) -> Result<DotrainOrderGui, GuiError> {
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

        let dotrain = state.sources[0].clone();
        let settings = if state.sources.len() > 1 {
            Some(state.sources[1..].to_vec())
        } else {
            None
        };
        let dotrain_order = DotrainOrder::new(dotrain, settings).await?;

        Ok(DotrainOrderGui {
            dotrain_order,
            selected_deployment: state.selected_deployment,
            field_values,
            deposits,
        })
    }

    #[wasm_bindgen(js_name = "clearState")]
    pub fn clear_state(&mut self) {
        self.field_values.clear();
        self.deposits.clear();
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
