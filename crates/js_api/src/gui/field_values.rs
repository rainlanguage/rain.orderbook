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
    /// Saves a value for a specific field binding with automatic preset detection.
    ///
    /// This function stores the provided value and automatically determines if it matches
    /// a preset from the field definition. Preset detection enables the UI to show
    /// whether a standard option or custom value is being used.
    ///
    /// # Parameters
    ///
    /// - `binding` - Field binding identifier from the YAML configuration
    /// - `value` - Value to save (can be a preset value or custom input)
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Value saved successfully and state callback triggered
    /// - `Err(FieldBindingNotFound)` - If the binding doesn't exist in the deployment
    ///
    /// # Preset Detection
    ///
    /// The function checks if the value matches any preset in the field definition.
    /// If it matches, it stores the preset index; otherwise, stores the raw value.
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// // Save a custom value
    /// const result = gui.saveFieldValue("max-price", "1500.50");
    /// if (result.error) {
    ///   console.error("Save failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
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

    /// Batch saves multiple field values in a single operation.
    ///
    /// This is more efficient than calling saveFieldValue multiple times, especially
    /// when loading saved configurations or applying templates. Each value is processed
    /// with the same preset detection as individual saves.
    ///
    /// # Parameters
    ///
    /// - `field_values` - Array of field-value pairs to save
    ///
    /// # Returns
    ///
    /// - `Ok(())` - All values saved successfully
    /// - `Err(GuiError)` - If any binding is invalid (partial save may occur)
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// const fields = [
    ///   { binding: "max-price", value: "1500" },
    ///   { binding: "min-amount", value: "100" },
    ///   { binding: "slippage", value: "0.5" }
    /// ];
    ///
    /// const result = gui.saveFieldValues(fields);
    /// if (result.error) {
    ///   console.error("Batch save failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "saveFieldValues", unchecked_return_type = "void")]
    pub fn save_field_values(&mut self, field_values: Vec<FieldValuePair>) -> Result<(), GuiError> {
        for field_value in field_values {
            self.save_field_value(field_value.binding, field_value.value)?;
        }
        Ok(())
    }

    /// Removes a previously saved field value.
    ///
    /// Use this to clear a field value, returning it to an unset state.
    ///
    /// # Parameters
    ///
    /// - `binding` - Field binding identifier to remove
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Value removed and state callback triggered
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// // Clear a field value
    /// const result = gui.removeFieldValue("max-price");
    /// if (result.error) {
    ///   console.error("Remove failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "removeFieldValue", unchecked_return_type = "void")]
    pub fn remove_field_value(&mut self, binding: String) -> Result<(), GuiError> {
        self.field_values.remove(&binding);
        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Retrieves a field value with preset expansion and metadata.
    ///
    /// This function returns the saved value along with information about whether it's a preset.
    ///
    /// # Parameters
    ///
    /// - `binding` - Field binding identifier to retrieve
    ///
    /// # Returns
    ///
    /// - `Ok(FieldValue)` - Field value with binding, value, and preset flag
    /// - `Err(FieldBindingNotFound)` - If no value has been saved for this binding
    /// - `Err(InvalidPreset)` - If saved preset index is invalid
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// const result = gui.getFieldValue("max-price");
    /// if (result.error) {
    ///   console.error("Not found:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const { binding, value, isPreset } = result.value;
    /// // binding is the field binding identifier
    /// // value is the field value
    /// // isPreset is a boolean indicating if the value is a preset
    /// ```
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

    /// Gets all configured field values with their metadata.
    ///
    /// Returns all field values that have been set, with preset values expanded.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<FieldValue>)` - Array of all configured field values
    /// - `Err(GuiError)` - If any field value retrieval fails
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// const result = gui.getAllFieldValues();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const [fieldValue1, fieldValue2, ...] = result.value;
    /// const {
    ///   // binding is the field binding identifier
    ///   binding,
    ///   // value is the field value
    ///   value,
    ///   // isPreset is a boolean indicating if the value is a preset
    ///   isPreset
    /// } = fieldValue1;
    /// ```
    #[wasm_export(js_name = "getAllFieldValues", unchecked_return_type = "FieldValue[]")]
    pub fn get_all_field_values(&self) -> Result<Vec<FieldValue>, GuiError> {
        let mut result = Vec::new();
        for (binding, _) in self.field_values.iter() {
            let field_value = self.get_field_value(binding.clone())?;
            result.push(field_value);
        }
        Ok(result)
    }

    /// Gets the complete definition for a specific field including presets and validation.
    ///
    /// Use this to build dynamic UIs that adapt to field configurations, showing
    /// appropriate input controls, preset options, and validation rules.
    ///
    /// # Parameters
    ///
    /// - `binding` - Field binding identifier to look up
    ///
    /// # Returns
    ///
    /// - `Ok(GuiFieldDefinitionCfg)` - Complete field configuration
    /// - `Err(FieldBindingNotFound)` - If binding doesn't exist in deployment
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// const result = gui.getFieldDefinition("max-price");
    /// if (result.error) {
    ///   console.error("Not found:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const {
    ///   // binding is the field binding identifier
    ///   binding,
    ///   // name is the display name for the field
    ///   name,
    ///   // description is the help text for the field
    ///   description,
    ///   // presets are the available preset options for the field
    ///   presets,
    ///   // default is the default value for the field if no value is set by the user
    ///   default,
    ///   // showCustomField is a boolean indicating if the field allows custom input
    ///   showCustomField
    /// } = result.value;
    /// ```
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

    /// Gets all field definitions with optional filtering by default value presence.
    ///
    /// This method helps build dynamic forms by providing all field configurations
    /// at once. The filter option allows separating required fields (no default)
    /// from optional fields (with default).
    ///
    /// # Parameters
    ///
    /// - `filter_defaults` - Optional filter:
    ///   - `Some(true)` - Only fields with default values
    ///   - `Some(false)` - Only fields without default values (required)
    ///   - `None` - All fields
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<GuiFieldDefinitionCfg>)` - Filtered field definitions
    /// - `Err(GuiError)` - If deployment configuration is invalid
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// // Get all fields
    /// const allFieldsResult = gui.getAllFieldDefinitions();
    /// if (allFieldsResult.error) {
    ///   console.error("Error:", allFieldsResult.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // Get only required fields (no defaults)
    /// const requiredResult = gui.getAllFieldDefinitions(false);
    /// if (requiredResult.error) {
    ///   console.error("Error:", requiredResult.error.readableMsg);
    ///   return;
    /// }
    /// const requiredFields = requiredResult.value;
    ///
    /// // Get optional fields (with defaults)
    /// const optionalResult = gui.getAllFieldDefinitions(true);
    /// if (optionalResult.error) {
    ///   console.error("Error:", optionalResult.error.readableMsg);
    ///   return;
    /// }
    /// const optionalFields = optionalResult.value;
    ///
    /// ```
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

    /// Lists field names that haven't been configured yet.
    ///
    /// Returns human-readable field names (not bindings) for fields that need values.
    /// Use this for validation and to guide users through required configurations.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<String>)` - Array of field names that need configuration
    /// - `Err(GuiError)` - If deployment configuration is invalid
    ///
    /// # Note
    ///
    /// This returns field names (e.g., "Maximum Price") not bindings (e.g., "max-price")
    /// for better user experience.
    ///
    /// # JavaScript Examples
    ///
    /// ```javascript
    /// const result = gui.getMissingFieldValues();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const missing = result.value;
    /// // Do something with the missing
    /// ```
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
        let mut gui = initialize_gui(None).await;

        gui.save_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        gui.save_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();

        let field_value = gui.get_field_value("binding-1".to_string()).unwrap();
        assert_eq!(field_value.binding, "binding-1");
        assert_eq!(field_value.value, "some-default-value");
        assert!(!field_value.is_preset);

        let field_value = gui.get_field_value("binding-2".to_string()).unwrap();
        assert_eq!(field_value.binding, "binding-2");
        assert_eq!(field_value.value, "99.2");
        assert!(field_value.is_preset);
    }

    #[wasm_bindgen_test]
    async fn test_set_get_all_field_values() {
        let mut gui = initialize_gui(None).await;

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
        assert!(!field_values[0].is_preset);
        assert_eq!(field_values[1].binding, "binding-2");
        assert_eq!(field_values[1].value, "99.2");
        assert!(field_values[1].is_preset);
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
        let mut gui = initialize_gui(None).await;

        gui.save_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        gui.get_field_value("binding-1".to_string()).unwrap();

        gui.remove_field_value("binding-1".to_string()).unwrap();
        let err = gui.get_field_value("binding-1".to_string()).unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldBindingNotFound("binding-1".to_string()).to_string()
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_field_definition() {
        let gui = initialize_gui(None).await;

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
        let gui = initialize_gui(None).await;

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
        let mut gui = initialize_gui(None).await;

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
        let mut gui = initialize_gui(None).await;

        let err = gui.check_field_values().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );

        gui.save_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();
        let res = gui.check_field_values();
        assert!(res.is_ok());
    }
}
