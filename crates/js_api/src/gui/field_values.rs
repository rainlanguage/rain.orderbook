use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FieldValuePair {
    binding: String,
    value: String,
}
impl_wasm_traits!(FieldValuePair);

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "saveFieldValue")]
    pub fn save_field_value(&mut self, binding: String, value: String) -> Result<(), GuiError> {
        self.deployment
            .fields
            .iter()
            .find(|field| field.binding == binding)
            .ok_or(GuiError::FieldBindingNotFound(binding.clone()))?;
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

    #[wasm_bindgen(js_name = "getFieldValue")]
    pub fn get_field_value(&self, binding: String) -> Result<String, GuiError> {
        let field_value = self
            .field_values
            .get(&binding)
            .ok_or(GuiError::FieldBindingNotFound(binding))?;
        Ok(field_value.clone())
    }

    #[wasm_bindgen(js_name = "getAllFieldValues")]
    pub fn get_all_field_values(&self) -> Vec<FieldValuePair> {
        self.field_values
            .iter()
            .map(|(k, v)| FieldValuePair {
                binding: k.clone(),
                value: v.clone(),
            })
            .collect()
    }

    #[wasm_bindgen(js_name = "getFieldDefinition")]
    pub fn get_field_definition(&self, binding: String) -> Result<GuiFieldDefinition, GuiError> {
        let field_definition = self
            .deployment
            .fields
            .iter()
            .find(|field| field.binding == binding)
            .ok_or(GuiError::FieldBindingNotFound(binding))?;
        Ok(field_definition.clone())
    }

    #[wasm_bindgen(js_name = "getAllFieldDefinitions")]
    pub fn get_all_field_definitions(&self) -> Vec<GuiFieldDefinition> {
        self.deployment.fields.clone()
    }
}
