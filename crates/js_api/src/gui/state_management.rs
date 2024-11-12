use super::*;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    config_hash: String,
    field_values: BTreeMap<String, String>,
    deposits: Vec<TokenDeposit>,
}

fn compute_config_hash(gui: &DotrainOrderGui) -> String {
    let config = gui.get_gui_config();
    let bytes = bincode::serialize(&config).expect("Failed to serialize config");
    let hash = Sha256::digest(&bytes);
    format!("{:x}", hash)
}

pub fn serialize(gui: &DotrainOrderGui) -> Result<String, GuiError> {
    let config_hash = compute_config_hash(gui);

    let state = SerializedGuiState {
        config_hash,
        field_values: gui.field_values.clone(),
        deposits: gui.deposits.clone(),
    };
    let bytes = bincode::serialize(&state)?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes)?;
    let compressed = encoder.finish()?;

    Ok(URL_SAFE.encode(compressed))
}

pub fn deserialize_state(gui: &mut DotrainOrderGui, serialized: String) -> Result<(), GuiError> {
    let compressed = URL_SAFE.decode(serialized)?;

    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;

    let state: SerializedGuiState = bincode::deserialize(&bytes)?;
    gui.field_values = state.field_values;
    gui.deposits = state.deposits;

    if state.config_hash != compute_config_hash(gui) {
        return Err(GuiError::DeserializedConfigMismatch);
    }

    Ok(())
}

pub fn clear_state(gui: &mut DotrainOrderGui) {
    gui.field_values.clear();
    gui.deposits.clear();
}
