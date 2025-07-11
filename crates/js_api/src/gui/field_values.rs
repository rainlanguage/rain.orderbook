use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]

pub struct FieldValuePair {
    field: String,
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
    pub field: String,
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
                    self.set_field_value(field.binding.clone(), default_value.clone())?;
                }
                None => return Err(GuiError::FieldValueNotSet(field.name.clone())),
            }
        }
        Ok(())
    }
}

#[wasm_export]
impl DotrainOrderGui {
    /// Sets a value for a specific field binding with automatic preset detection.
    ///
    /// This function stores the provided value and automatically determines if it matches
    /// a preset from the field definition. Preset detection enables the UI to show
    /// whether a standard option or custom value is being used.
    ///
    /// ## Preset Detection
    ///
    /// The function checks if the value matches any preset in the field definition.
    /// If it matches, it stores the preset index; otherwise, stores the raw value.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Set a custom value
    /// const result = gui.setFieldValue("max-price", "1500.50");
    /// if (result.error) {
    ///   console.error("Set failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "setFieldValue", unchecked_return_type = "void")]
    pub fn set_field_value(
        &mut self,
        #[wasm_export(param_description = "Field identifier from the YAML configuration")]
        field: String,
        #[wasm_export(param_description = "Value to save (can be a preset value or custom input)")]
        value: String,
    ) -> Result<(), GuiError> {
        let field_definition = self.get_field_definition(&field)?;

        if let Some(validation) = &field_definition.validation {
            validation::validate_field_value(&field_definition.name, &value, validation)?;
        }

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

        self.field_values.insert(field, value);

        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Batch sets multiple field values in a single operation.
    ///
    /// This is more efficient than calling setFieldValue multiple times, especially
    /// when loading saved configurations or applying templates. Each value is processed
    /// with the same preset detection as individual saves.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const fields = [
    ///   { field: "max-price", value: "1500" },
    ///   { field: "min-amount", value: "100" },
    ///   { field: "slippage", value: "0.5" }
    /// ];
    ///
    /// const result = gui.setFieldValues(fields);
    /// if (result.error) {
    ///   console.error("Batch set failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "setFieldValues", unchecked_return_type = "void")]
    pub fn set_field_values(
        &mut self,
        #[wasm_export(param_description = "Array of field-value pairs to save")] field_values: Vec<
            FieldValuePair,
        >,
    ) -> Result<(), GuiError> {
        for field_value in field_values {
            self.set_field_value(field_value.field, field_value.value)?;
        }
        Ok(())
    }

    /// Unsets a previously set field value.
    ///
    /// Use this to clear a field value, returning it to an unset state.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Clear a field value
    /// const result = gui.unsetFieldValue("max-price");
    /// if (result.error) {
    ///   console.error("Unset failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "unsetFieldValue", unchecked_return_type = "void")]
    pub fn unset_field_value(
        &mut self,
        #[wasm_export(param_description = "Field identifier from the YAML configuration")]
        field: String,
    ) -> Result<(), GuiError> {
        self.field_values.remove(&field);
        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Retrieves a field value with preset expansion and metadata.
    ///
    /// This function returns the saved value along with information about whether it's a preset.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getFieldValue("max-price");
    /// if (result.error) {
    ///   console.error("Not found:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const { field, value, isPreset } = result.value;
    /// // field is the field identifier
    /// // value is the field value
    /// // isPreset is a boolean indicating if the value is a preset
    /// ```
    #[wasm_export(
        js_name = "getFieldValue",
        unchecked_return_type = "FieldValue",
        return_description = "Field value with the identifier, value, and preset flag"
    )]
    pub fn get_field_value(
        &self,
        #[wasm_export(param_description = "Field identifier from the YAML configuration")]
        field: String,
    ) -> Result<FieldValue, GuiError> {
        let field_value = self
            .field_values
            .get(&field)
            .ok_or(GuiError::FieldBindingNotFound(field.clone()))?;
        let preset = match field_value.is_preset {
            true => {
                let field_definition = self.get_field_definition(&field)?;
                let presets = field_definition
                    .presets
                    .ok_or(GuiError::BindingHasNoPresets(field.clone()))?;
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
            field: field.clone(),
            value: preset.value.clone(),
            is_preset: field_value.is_preset,
        })
    }

    /// Gets all configured field values with their metadata.
    ///
    /// Returns all field values that have been set, with preset values expanded.
    ///
    /// ## Examples
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
    ///   // field is the field identifier
    ///   field,
    ///   // value is the field value
    ///   value,
    ///   // isPreset is a boolean indicating if the value is a preset
    ///   isPreset
    /// } = fieldValue1;
    /// ```
    #[wasm_export(
        js_name = "getAllFieldValues",
        unchecked_return_type = "FieldValue[]",
        return_description = "Array of all configured field values"
    )]
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
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getFieldDefinition("max-price");
    /// if (result.error) {
    ///   console.error("Not found:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const {
    ///   // field is the field identifier
    ///   field,
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
        unchecked_return_type = "GuiFieldDefinitionCfg",
        return_description = "Complete field configuration"
    )]
    pub fn get_field_definition(
        &self,
        #[wasm_export(param_description = "Field binding identifier to look up")] field: &str,
    ) -> Result<GuiFieldDefinitionCfg, GuiError> {
        let deployment = self.get_current_deployment()?;
        let field_definition = deployment
            .fields
            .iter()
            .find(|field_definition| field_definition.binding == field)
            .ok_or(GuiError::FieldBindingNotFound(field.to_string()))?;
        Ok(field_definition.clone())
    }

    /// Gets all field definitions with optional filtering by default value presence.
    ///
    /// This method helps build dynamic forms by providing all field configurations
    /// at once. The filter option allows separating required fields (no default)
    /// from optional fields (with default).
    ///
    /// ## Examples
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
        unchecked_return_type = "GuiFieldDefinitionCfg[]",
        return_description = "Filtered field definitions"
    )]
    pub fn get_all_field_definitions(
        &self,
        #[wasm_export(
            param_description = "Optional filter: **true** for fields with defaults, **false** for fields without defaults, **undefined** for all"
        )]
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

    /// Lists field definitions that haven't been configured yet.
    ///
    /// Returns field definitions for fields that need values.
    /// Use this for validation and to guide users through required configurations.
    ///
    /// ## Examples
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
    #[wasm_export(
        js_name = "getMissingFieldValues",
        unchecked_return_type = "GuiFieldDefinitionCfg[]",
        return_description = "Array of field definitions that need to be set"
    )]
    pub fn get_missing_field_values(&self) -> Result<Vec<GuiFieldDefinitionCfg>, GuiError> {
        let deployment = self.get_current_deployment()?;
        let mut missing_field_values = Vec::new();

        for field in deployment.fields.iter() {
            if !self.field_values.contains_key(&field.binding) {
                missing_field_values.push(field.clone());
            }
        }
        Ok(missing_field_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::tests::{initialize_gui, initialize_validation_gui};
    use crate::gui::validation;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_set_get_field_value() {
        let mut gui = initialize_gui(None).await;

        gui.set_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();

        let field_value = gui.get_field_value("binding-1".to_string()).unwrap();
        assert_eq!(field_value.field, "binding-1");
        assert_eq!(field_value.value, "some-default-value");
        assert!(!field_value.is_preset);

        let field_value = gui.get_field_value("binding-2".to_string()).unwrap();
        assert_eq!(field_value.field, "binding-2");
        assert_eq!(field_value.value, "99.2");
        assert!(field_value.is_preset);
    }

    #[wasm_bindgen_test]
    async fn test_set_get_all_field_values() {
        let mut gui = initialize_gui(None).await;

        gui.set_field_values(vec![
            FieldValuePair {
                field: "binding-1".to_string(),
                value: "some-default-value".to_string(),
            },
            FieldValuePair {
                field: "binding-2".to_string(),
                value: "99.2".to_string(),
            },
        ])
        .unwrap();

        let field_values = gui.get_all_field_values().unwrap();
        assert_eq!(field_values.len(), 2);
        assert_eq!(field_values[0].field, "binding-1");
        assert_eq!(field_values[0].value, "some-default-value");
        assert!(!field_values[0].is_preset);
        assert_eq!(field_values[1].field, "binding-2");
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
            validation: None,
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
            validation: None,
        }
    }

    #[wasm_bindgen_test]
    async fn test_unset_field_value() {
        let mut gui = initialize_gui(None).await;

        gui.set_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();
        gui.get_field_value("binding-1".to_string()).unwrap();

        gui.unset_field_value("binding-1".to_string()).unwrap();
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
        assert_eq!(field_values[0], get_binding_1());
        assert_eq!(field_values[1], get_binding_2());

        gui.set_field_value("binding-1".to_string(), "some-default-value".to_string())
            .unwrap();

        let field_values = gui.get_missing_field_values().unwrap();
        assert_eq!(field_values.len(), 1);
        assert_eq!(field_values[0], get_binding_2());
    }

    #[wasm_bindgen_test]
    async fn test_check_field_values() {
        let mut gui = initialize_gui(None).await;

        let err = gui.check_field_values().unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );

        gui.set_field_value("binding-2".to_string(), "99.2".to_string())
            .unwrap();
        let res = gui.check_field_values();
        assert!(res.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_number_minimum_maximum() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("price-field".to_string(), "50.00".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("price-field".to_string(), "5.00".to_string());
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::BelowMinimum {
                name,
                value,
                minimum,
            })) => {
                assert_eq!(name, "Price Field");
                assert_eq!(value, "5.00");
                assert_eq!(minimum, "10");
            }
            _ => panic!("Expected BelowMinimum error"),
        }

        let result = gui.save_field_value("price-field".to_string(), "1500.00".to_string());
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::AboveMaximum {
                name,
                value,
                maximum,
            })) => {
                assert_eq!(name, "Price Field");
                assert_eq!(value, "1500.00");
                assert_eq!(maximum, "1000");
            }
            _ => panic!("Expected AboveMaximum error"),
        }

        let result = gui.save_field_value("price-field".to_string(), "10".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("price-field".to_string(), "1000".to_string());
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_number_multiple_of() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("price-field".to_string(), "99.99".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("price-field".to_string(), "99.999".to_string());
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::NotMultipleOf {
                name,
                value,
                multiple_of,
            })) => {
                assert_eq!(name, "Price Field");
                assert_eq!(value, "99.999");
                assert_eq!(multiple_of, "0.01");
            }
            _ => panic!("Expected NotMultipleOf error"),
        }
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_number_exclusive_bounds() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("quantity-field".to_string(), "0".to_string());
        match result {
            Err(GuiError::ValidationError(
                validation::GuiValidationError::BelowExclusiveMinimum {
                    name,
                    value,
                    exclusive_minimum,
                },
            )) => {
                assert_eq!(name, "Quantity Field");
                assert_eq!(value, "0");
                assert_eq!(exclusive_minimum, "0");
            }
            _ => panic!("Expected BelowExclusiveMinimum error"),
        }

        let result = gui.save_field_value("quantity-field".to_string(), "0.001".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("quantity-field".to_string(), "100000".to_string());
        match result {
            Err(GuiError::ValidationError(
                validation::GuiValidationError::AboveExclusiveMaximum {
                    name,
                    value,
                    exclusive_maximum,
                },
            )) => {
                assert_eq!(name, "Quantity Field");
                assert_eq!(value, "100000");
                assert_eq!(exclusive_maximum, "100000");
            }
            _ => panic!("Expected AboveExclusiveMaximum error"),
        }

        let result = gui.save_field_value("quantity-field".to_string(), "99999.999".to_string());
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_number_complex_constraints() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("percentage-field".to_string(), "50.5".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("percentage-field".to_string(), "50.55".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::NotMultipleOf { .. }
            ))
        ));

        let result = gui.save_field_value("percentage-field".to_string(), "0".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("percentage-field".to_string(), "100".to_string());
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_number_invalid_formats() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("simple-number".to_string(), "".to_string());
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::InvalidNumber {
                name,
                value,
            })) => {
                assert_eq!(name, "Simple Number");
                assert_eq!(value, "");
            }
            _ => panic!("Expected InvalidNumber error"),
        }

        let result = gui.save_field_value("simple-number".to_string(), "abc".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::InvalidNumber { .. }
            ))
        ));

        let result = gui.save_field_value("simple-number".to_string(), "12.34.56".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::InvalidNumber { .. }
            ))
        ));

        let result = gui.save_field_value("simple-number".to_string(), "1e10".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::InvalidNumber { .. }
            ))
        ));

        let result = gui.save_field_value("simple-number".to_string(), "١٢٣".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::InvalidNumber { .. }
            ))
        ));
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_string_length_constraints() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("username-field".to_string(), "john_doe".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("username-field".to_string(), "jo".to_string());
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::StringTooShort {
                name,
                length,
                minimum,
            })) => {
                assert_eq!(name, "Username");
                assert_eq!(length, 2);
                assert_eq!(minimum, 3);
            }
            _ => panic!("Expected StringTooShort error"),
        }

        let result = gui.save_field_value(
            "username-field".to_string(),
            "this_username_is_way_too_long".to_string(),
        );
        match result {
            Err(GuiError::ValidationError(validation::GuiValidationError::StringTooLong {
                name,
                length,
                maximum,
            })) => {
                assert_eq!(name, "Username");
                assert_eq!(length, 29);
                assert_eq!(maximum, 20);
            }
            _ => panic!("Expected StringTooLong error"),
        }

        let result = gui.save_field_value("username-field".to_string(), "abc".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("username-field".to_string(), "a".repeat(20));
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_string_edge_cases() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("description-field".to_string(), "".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("code-field".to_string(), "".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::StringTooShort { .. }
            ))
        ));

        let result = gui.save_field_value("description-field".to_string(), "a".repeat(500));
        assert!(result.is_ok());

        let result = gui.save_field_value("description-field".to_string(), "a".repeat(501));
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::StringTooLong { .. }
            ))
        ));

        let result =
            gui.save_field_value("username-field".to_string(), "🦀🦀🦀rust🦀🦀🦀".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::StringTooLong { .. }
            ))
        ));

        let result = gui.save_field_value(
            "any-string".to_string(),
            "Any value at all!@#$%^&*()".to_string(),
        );
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_boolean() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("enabled-field".to_string(), "1".to_string());
        assert!(result.is_ok());

        let result = gui.save_field_value("enabled-field".to_string(), "0".to_string());
        assert!(result.is_ok());

        let test_cases = vec![
            "True", "FALSE", "yes", "no", "true", "false", "on", "off", "", " true", "true ",
        ];

        for test_value in test_cases {
            let result = gui.save_field_value("enabled-field".to_string(), test_value.to_string());
            match result {
                Err(GuiError::ValidationError(
                    validation::GuiValidationError::InvalidBoolean { name, value },
                )) => {
                    assert_eq!(name, "Enabled");
                    assert_eq!(value, test_value);
                }
                _ => panic!("Expected InvalidBoolean error for value: {}", test_value),
            }
        }
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_preset_with_validation() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("preset-number-field".to_string(), "100".to_string());
        assert!(result.is_ok());
        let field_value = gui
            .get_field_value("preset-number-field".to_string())
            .unwrap();
        assert!(field_value.is_preset);
        assert_eq!(field_value.value, "100");

        let result = gui.save_field_value("preset-number-field".to_string(), "120".to_string());
        assert!(result.is_ok());
        let field_value = gui
            .get_field_value("preset-number-field".to_string())
            .unwrap();
        assert!(!field_value.is_preset);
        assert_eq!(field_value.value, "120");

        let result = gui.save_field_value("preset-number-field".to_string(), "125".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::NotMultipleOf { .. }
            ))
        ));

        let result = gui.save_field_value("preset-number-field".to_string(), "5".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::BelowMinimum { .. }
            ))
        ));
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_string_preset_with_validation() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value("preset-string-field".to_string(), "alpha".to_string());
        assert!(result.is_ok());
        let field_value = gui
            .get_field_value("preset-string-field".to_string())
            .unwrap();
        assert!(field_value.is_preset);

        let result = gui.save_field_value("preset-string-field".to_string(), "custom".to_string());
        assert!(result.is_ok());
        let field_value = gui
            .get_field_value("preset-string-field".to_string())
            .unwrap();
        assert!(!field_value.is_preset);

        let result = gui.save_field_value("preset-string-field".to_string(), "xyz".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::StringTooShort { .. }
            ))
        ));

        let result = gui.save_field_value(
            "preset-string-field".to_string(),
            "verylongvalue".to_string(),
        );
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::StringTooLong { .. }
            ))
        ));
    }

    #[wasm_bindgen_test]
    async fn test_save_field_value_no_validation() {
        let mut gui = initialize_validation_gui().await;

        let test_values = vec![
            "123",
            "abc",
            "true",
            "false",
            "",
            "!@#$%^&*()",
            "very long string with many characters",
            "0",
            "🦀 Rust 🦀",
        ];

        for value in test_values {
            let result = gui.save_field_value("no-validation-field".to_string(), value.to_string());
            assert!(result.is_ok(), "Failed to save value: {}", value);

            let field_value = gui
                .get_field_value("no-validation-field".to_string())
                .unwrap();
            assert_eq!(field_value.value, value);
        }
    }

    #[wasm_bindgen_test]
    async fn test_save_field_values_batch_with_validation() {
        let mut gui = initialize_validation_gui().await;

        let valid_batch = vec![
            FieldValuePair {
                binding: "price-field".to_string(),
                value: "100.00".to_string(),
            },
            FieldValuePair {
                binding: "username-field".to_string(),
                value: "valid_user".to_string(),
            },
            FieldValuePair {
                binding: "enabled-field".to_string(),
                value: "1".to_string(),
            },
        ];

        let result = gui.save_field_values(valid_batch);
        assert!(result.is_ok());

        let invalid_batch = vec![
            FieldValuePair {
                binding: "price-field".to_string(),
                value: "100.00".to_string(),
            },
            FieldValuePair {
                binding: "username-field".to_string(),
                value: "ab".to_string(), // Too short
            },
            FieldValuePair {
                binding: "enabled-field".to_string(),
                value: "true".to_string(),
            },
        ];

        let result = gui.save_field_values(invalid_batch);
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::StringTooShort { .. }
            ))
        ));

        let field_result = gui.get_field_value("price-field".to_string());
        assert!(field_result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_very_precise_decimal_validation() {
        let mut gui = initialize_validation_gui().await;

        let result = gui.save_field_value(
            "simple-number".to_string(),
            "0.123456789012345678".to_string(),
        );
        assert!(result.is_ok());

        let result = gui.save_field_value("price-field".to_string(), "999.99".to_string());
        assert!(result.is_ok());

        let result =
            gui.save_field_value("price-field".to_string(), "100.00000000000001".to_string());
        assert!(matches!(
            result,
            Err(GuiError::ValidationError(
                validation::GuiValidationError::NotMultipleOf { .. }
            ))
        ));
    }
}
