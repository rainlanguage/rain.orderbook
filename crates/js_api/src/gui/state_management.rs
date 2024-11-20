use super::*;
use deposits::TokenDeposit;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    config_hash: String,
    field_values: BTreeMap<String, GuiPreset>,
    deposits: Vec<TokenDeposit>,
}

#[wasm_bindgen]
impl DotrainOrderGui {
    fn compute_config_hash(&self) -> String {
        let config = self.get_gui_config();
        let bytes = bincode::serialize(&config).expect("Failed to serialize config");
        let hash = Sha256::digest(&bytes);
        format!("{:x}", hash)
    }

    #[wasm_bindgen(js_name = "serializeState")]
    pub fn serialize(&self) -> Result<String, GuiError> {
        let config_hash = self.compute_config_hash();

        let mut field_values = BTreeMap::new();
        for (k, v) in self.field_values.iter() {
            let preset = if v.is_preset {
                let field_definition = self.get_field_definition(k)?;
                let preset = field_definition
                    .presets
                    .iter()
                    .find(|preset| preset.id == v.value)
                    .ok_or(GuiError::InvalidPreset)?;
                preset.clone()
            } else {
                GuiPreset {
                    id: "".to_string(),
                    name: None,
                    value: v.value.clone(),
                }
            };
            field_values.insert(k.clone(), preset);
        }

        let state = SerializedGuiState {
            config_hash,
            field_values: field_values.clone(),
            deposits: self.deposits.clone(),
        };
        let bytes = bincode::serialize(&state)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let compressed = encoder.finish()?;

        Ok(URL_SAFE.encode(compressed))
    }

    #[wasm_bindgen(js_name = "deserializeState")]
    pub fn deserialize_state(&mut self, serialized: String) -> Result<(), GuiError> {
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

        self.field_values = field_values;
        self.deposits = state.deposits;

        if state.config_hash != self.compute_config_hash() {
            return Err(GuiError::DeserializedConfigMismatch);
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "clearState")]
    pub fn clear_state(&mut self) {
        self.field_values.clear();
        self.deposits.clear();
    }
}
