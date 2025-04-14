use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]

pub struct FieldValuePair {
    binding: String,
    value: String,
}
impl_wasm_traits!(FieldValuePair);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PairValue {
    pub is_preset: bool,
    pub value: String,
}
impl_wasm_traits!(PairValue);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct FieldValue {
    pub binding: String,
    pub value: String,
    pub is_preset: bool,
}
impl_wasm_traits!(FieldValue);

impl DotrainOrderGui {
    pub fn check_field_values(&mut self) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;

        for field in deployment.fields.iter() {
            if self.field_values.contains_key(&field.binding) {
                continue;
            }

            match &field.default {
                Some(default_value) => {
                    self.save_field_value(field.binding.clone(), default_value.clone())?;
                }
                None => return Err(GuiError::FieldValueNotSet(field.name.clone())),
            }
        }
        Ok(())
    }
}

#[wasm_export]
impl DotrainOrderGui {
    #[wasm_export(js_name = "saveFieldValue", unchecked_return_type = "void")]
    pub fn save_field_value(&mut self, binding: String, value: String) -> Result<(), GuiError> {
        let field_definition = self.get_field_definition(&binding)?;

        let value = match field_definition.presets.as_ref() {
            Some(presets) => match presets.iter().position(|p| p.value == value) {
                Some(index) => field_values::PairValue {
                    is_preset: true,
                    value: index.to_string(),
                },
                None => field_values::PairValue {
                    is_preset: false,
                    value,
                },
            },
            None => field_values::PairValue {
                is_preset: false,
                value,
            },
        };

        self.field_values.insert(binding, value);

        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "saveFieldValues", unchecked_return_type = "void")]
    pub fn save_field_values(&mut self, field_values: Vec<FieldValuePair>) -> Result<(), GuiError> {
        for field_value in field_values {
            self.save_field_value(field_value.binding, field_value.value)?;
        }
        Ok(())
    }

    #[wasm_export(js_name = "removeFieldValue", unchecked_return_type = "void")]
    pub fn remove_field_value(&mut self, binding: String) -> Result<(), GuiError> {
        self.field_values.remove(&binding);
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "getFieldValue", unchecked_return_type = "FieldValue")]
    pub fn get_field_value(&self, binding: String) -> Result<FieldValue, GuiError> {
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
            false => GuiPresetCfg {
                id: "".to_string(),
                name: None,
                value: field_value.value.clone(),
            },
        };
        Ok(FieldValue {
            binding: binding.clone(),
            value: preset.value.clone(),
            is_preset: field_value.is_preset,
        })
    }

    #[wasm_export(js_name = "getAllFieldValues", unchecked_return_type = "FieldValue[]")]
    pub fn get_all_field_values(&self) -> Result<Vec<FieldValue>, GuiError> {
        let mut result = Vec::new();
        for (binding, _) in self.field_values.iter() {
            let field_value = self.get_field_value(binding.clone())?;
            result.push(field_value);
        }
        Ok(result)
    }

    #[wasm_export(
        js_name = "getFieldDefinition",
        unchecked_return_type = "GuiFieldDefinitionCfg"
    )]
    pub fn get_field_definition(&self, binding: &str) -> Result<GuiFieldDefinitionCfg, GuiError> {
        let deployment = self.get_current_deployment()?;
        let field_definition = deployment
            .fields
            .iter()
            .find(|field| field.binding == binding)
            .ok_or(GuiError::FieldBindingNotFound(binding.to_string()))?;
        Ok(field_definition.clone())
    }

    /// Get all field definitions, optionally filtered by whether they have a default value.
    #[wasm_export(
        js_name = "getAllFieldDefinitions",
        unchecked_return_type = "GuiFieldDefinitionCfg[]"
    )]
    pub fn get_all_field_definitions(
        &self,
        filter_defaults: Option<bool>,
    ) -> Result<Vec<GuiFieldDefinitionCfg>, GuiError> {
        let deployment = self.get_current_deployment()?;
        let mut field_definitions = deployment.fields.clone();

        match filter_defaults {
            Some(true) => field_definitions.retain(|field| field.default.is_some()),
            Some(false) => field_definitions.retain(|field| field.default.is_none()),
            None => (),
        }
        Ok(field_definitions)
    }

    #[wasm_export(js_name = "getMissingFieldValues", unchecked_return_type = "string[]")]
    pub fn get_missing_field_values(&self) -> Result<Vec<String>, GuiError> {
        let deployment = self.get_current_deployment()?;
        let mut missing_field_values = Vec::new();

        for field in deployment.fields.iter() {
            if !self.field_values.contains_key(&field.binding) {
                missing_field_values.push(field.name.clone());
            }
        }
        Ok(missing_field_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::tests::initialize_gui;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_set_get_field_value() {
        let mut gui = initialize_gui().await;

        gui.save_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        gui.save_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();

        let field_value = gui.get_field_value("binding-1".to_string()).unwrap();
        assert_eq!(field_value.binding, "binding-1");
        assert_eq!(field_value.value, "some-default-value");
        assert_eq!(field_value.is_preset, false);

        let field_value = gui.get_field_value("binding-2".to_string()).unwrap();
        assert_eq!(field_value.binding, "binding-2");
        assert_eq!(field_value.value, "99.2");
        assert_eq!(field_value.is_preset, true);
    }

    #[wasm_bindgen_test]
    async fn test_set_get_all_field_values() {
        let mut gui = initialize_gui().await;

        gui.save_field_values(vec![
            FieldValuePair {
                binding: "binding-1".to_string(),
                value: "some-default-value".to_string(),
            },
            FieldValuePair {
                binding: "binding-2".to_string(),
                value: "99.2".to_string(),
            },
        ])
        .unwrap();

        let field_values = gui.get_all_field_values().unwrap();
        assert_eq!(field_values.len(), 2);
        assert_eq!(field_values[0].binding, "binding-1");
        assert_eq!(field_values[0].value, "some-default-value");
        assert_eq!(field_values[0].is_preset, false);
        assert_eq!(field_values[1].binding, "binding-2");
        assert_eq!(field_values[1].value, "99.2");
        assert_eq!(field_values[1].is_preset, true);
    }

    fn get_binding_1() -> GuiFieldDefinitionCfg {
        GuiFieldDefinitionCfg {
            binding: String::from("binding-1"),
            name: String::from("Field 1 name"),
            description: Some(String::from("Field 1 description")),
            presets: Some(Vec::from([
                GuiPresetCfg {
                    id: String::from("0"),
                    name: Some(String::from("Preset 1")),
                    value: String::from("0x1234567890abcdef1234567890abcdef12345678"),
                },
                GuiPresetCfg {
                    id: String::from("1"),
                    name: Some(String::from("Preset 2")),
                    value: String::from("false"),
                },
                GuiPresetCfg {
                    id: String::from("2"),
                    name: Some(String::from("Preset 3")),
                    value: String::from("some-string"),
                },
            ])),
            default: Some(String::from("some-default-value")),
            show_custom_field: None,
        }
    }

    fn get_binding_2() -> GuiFieldDefinitionCfg {
        GuiFieldDefinitionCfg {
            binding: String::from("binding-2"),
            name: String::from("Field 2 name"),
            description: Some(String::from("Field 2 description")),
            presets: Some(Vec::from([
                GuiPresetCfg {
                    id: String::from("0"),
                    name: None,
                    value: String::from("99.2"),
                },
                GuiPresetCfg {
                    id: String::from("1"),
                    name: None,
                    value: String::from("582.1"),
                },
                GuiPresetCfg {
                    id: String::from("2"),
                    name: None,
                    value: String::from("648.239"),
                },
            ])),
            default: None,
            show_custom_field: Some(true),
        }
    }

    #[wasm_bindgen_test]
    async fn test_remove_field_value() {
        let mut gui = initialize_gui().await;

        gui.save_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        gui.get_field_value("binding-1".to_string()).unwrap();

        gui.remove_field_value("binding-1".to_string()).unwrap();
        gui.get_field_value("binding-1".to_string()).unwrap_err();
    }

    #[wasm_bindgen_test]
    async fn test_get_field_definition() {
        let gui = initialize_gui().await;

        let field_definition = gui.get_field_definition("binding-1").unwrap();
        assert_eq!(field_definition, get_binding_1());

        let field_definition = gui.get_field_definition("binding-2").unwrap();
        assert_eq!(field_definition, get_binding_2());

        let err = gui.get_field_definition("invalid-binding").unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldBindingNotFound("invalid-binding".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The field binding 'invalid-binding' could not be found in the YAML configuration."
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_all_field_definitions() {
        let gui = initialize_gui().await;

        let field_definitions = gui.get_all_field_definitions(None).unwrap();
        assert_eq!(field_definitions.len(), 2);
        assert_eq!(field_definitions[0], get_binding_1());
        assert_eq!(field_definitions[1], get_binding_2());

        let field_definitions = gui.get_all_field_definitions(Some(true)).unwrap();
        assert_eq!(field_definitions.len(), 1);
        assert_eq!(field_definitions[0], get_binding_1());

        let field_definitions = gui.get_all_field_definitions(Some(false)).unwrap();
        assert_eq!(field_definitions.len(), 1);
        assert_eq!(field_definitions[0], get_binding_2());
    }

    #[wasm_bindgen_test]
    async fn test_get_missing_field_values() {
        let mut gui = initialize_gui().await;

        let field_values = gui.get_missing_field_values().unwrap();
        assert_eq!(field_values.len(), 2);
        assert_eq!(field_values[0], "Field 1 name");
        assert_eq!(field_values[1], "Field 2 name");

        gui.save_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();

        let field_values = gui.get_missing_field_values().unwrap();
        assert_eq!(field_values.len(), 1);
        assert_eq!(field_values[0], "Field 2 name");
    }

    #[wasm_bindgen_test]
    async fn test_check_field_values() {
        let mut gui = initialize_gui().await;

        let err = gui.check_field_values().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );

        gui.save_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();
        gui.check_field_values().unwrap();
    }
}
