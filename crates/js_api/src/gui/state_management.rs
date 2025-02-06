use super::*;
use rain_orderbook_app_settings::token::Token;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    field_values: BTreeMap<String, GuiPreset>,
    deposits: BTreeMap<String, GuiPreset>,
    select_tokens: BTreeMap<String, Token>,
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

    #[wasm_bindgen(js_name = "serializeState")]
    pub fn serialize_state(&self) -> Result<String, GuiError> {
        let mut field_values = BTreeMap::new();
        for (k, v) in self.field_values.iter() {
            let preset = if v.is_preset {
                let presets = Gui::parse_field_presets(
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

        let mut select_tokens: BTreeMap<String, Token> = BTreeMap::new();
        if let Some(st) = Gui::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &self.selected_deployment,
        )? {
            for key in st {
                if let Ok(token) = self.dotrain_order.orderbook_yaml().get_token(&key) {
                    select_tokens.insert(key, token);
                }
            }
        }

        let order_key = Deployment::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        let mut vault_ids = BTreeMap::new();
        for (i, vault_id) in Order::parse_vault_ids(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &order_key,
            true,
        )?
        .iter()
        .enumerate()
        {
            vault_ids.insert((true, i as u8), vault_id.as_ref().map(|v| v.to_string()));
        }
        for (i, vault_id) in Order::parse_vault_ids(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &order_key,
            false,
        )?
        .iter()
        .enumerate()
        {
            vault_ids.insert((false, i as u8), vault_id.as_ref().map(|v| v.to_string()));
        }

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

        let dotrain_order_gui = DotrainOrderGui {
            dotrain_order,
            field_values,
            deposits,
            selected_deployment: state.selected_deployment.clone(),
        };

        let deployment_select_tokens = Gui::parse_select_tokens(
            dotrain_order_gui.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for (key, token) in state.select_tokens {
            let select_tokens = deployment_select_tokens
                .as_ref()
                .ok_or(GuiError::SelectTokensNotSet)?;
            if !select_tokens.contains(&key) {
                return Err(GuiError::TokenNotInSelectTokens(key));
            }
            if dotrain_order_gui.is_select_token_set(key.clone())? {
                Token::remove_record_from_yaml(
                    dotrain_order_gui.dotrain_order.orderbook_yaml().documents,
                    &key,
                )?;
            }
            Token::add_record_to_yaml(
                dotrain_order_gui.dotrain_order.orderbook_yaml().documents,
                &key,
                &token.network.key,
                &token.address.to_string(),
                token.decimals.map(|d| d.to_string()).as_deref(),
                token.label.map(|l| l.to_string()).as_deref(),
                token.symbol.map(|s| s.to_string()).as_deref(),
            )?;
        }

        let order_key = Deployment::parse_order_key(
            dotrain_order_gui.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for ((is_input, index), vault_id) in state.vault_ids {
            dotrain_order_gui
                .dotrain_order
                .dotrain_yaml()
                .get_order(&order_key)
                .and_then(|mut order| {
                    order.update_vault_id(is_input, index, vault_id.unwrap_or_default())
                })?;
        }

        Ok(dotrain_order_gui)
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
