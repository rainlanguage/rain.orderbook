use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]

pub struct FieldValuePair {
    binding: String,
    value: PairValue,
}
impl_all_wasm_traits!(FieldValuePair);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PairValue {
    pub is_preset: bool,
    pub value: String,
}
impl_all_wasm_traits!(PairValue);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AllFieldValuesResult {
    pub binding: String,
    pub value: GuiPreset,
}
impl_all_wasm_traits!(AllFieldValuesResult);

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "saveFieldValue")]
    pub fn save_field_value(&mut self, binding: String, value: PairValue) -> Result<(), GuiError> {
        let field_definition = self.get_field_definition(&binding)?;
        if value.is_preset {
            let presets = field_definition
                .presets
                .ok_or(GuiError::BindingHasNoPresets(binding.clone()))?;

            if !presets
                .iter()
                .find(|preset| preset.id == value.value)
                .is_some()
            {
                return Err(GuiError::InvalidPreset);
            }
        }
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

    #[wasm_bindgen(js_name = "removeFieldValue")]
    pub fn remove_field_value(&mut self, binding: String) {
        self.field_values.remove(&binding);
    }

    #[wasm_bindgen(js_name = "getFieldValue")]
    pub fn get_field_value(&self, binding: String) -> Result<GuiPreset, GuiError> {
        let field_value = self
            .field_values
            .get(&binding)
            .ok_or(GuiError::FieldBindingNotFound(binding.clone()))?;
        let preset = match field_value.is_preset {
            true => {
                let field_definition = self.get_field_definition(&binding)?;
                let presets = field_definition
                    .presets
                    .ok_or(GuiError::BindingHasNoPresets(binding.clone()))?;
                presets
                    .iter()
                    .find(|preset| preset.id == field_value.value)
                    .ok_or(GuiError::InvalidPreset)?
                    .clone()
            }
            false => GuiPreset {
                id: "".to_string(),
                name: None,
                value: field_value.value.clone(),
            },
        };
        Ok(preset)
    }

    #[wasm_bindgen(js_name = "getAllFieldValues")]
    pub fn get_all_field_values(&self) -> Result<Vec<AllFieldValuesResult>, GuiError> {
        let mut result = Vec::new();
        for (binding, _) in self.field_values.iter() {
            result.push(AllFieldValuesResult {
                binding: binding.clone(),
                value: self.get_field_value(binding.clone())?,
            });
        }
        Ok(result)
    }

    #[wasm_bindgen(js_name = "getFieldDefinition")]
    pub fn get_field_definition(&self, binding: &str) -> Result<GuiFieldDefinition, GuiError> {
        let field_definition = self
            .deployment
            .fields
            .iter()
            .find(|field| field.binding == binding)
            .ok_or(GuiError::FieldBindingNotFound(binding.to_string()))?;
        Ok(field_definition.clone())
    }

    #[wasm_bindgen(js_name = "getAllFieldDefinitions")]
    pub fn get_all_field_definitions(&self) -> Vec<GuiFieldDefinition> {
        self.deployment.fields.clone()
    }
}
