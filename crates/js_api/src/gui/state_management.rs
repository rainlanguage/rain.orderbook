use super::*;
use deposits::TokenDeposit;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    config_hash: String,
    field_values: BTreeMap<String, String>,
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

        let state = SerializedGuiState {
            config_hash,
            field_values: self.field_values.clone(),
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
        self.field_values = state.field_values;
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
