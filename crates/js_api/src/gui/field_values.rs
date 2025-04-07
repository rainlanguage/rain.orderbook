use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]

pub struct FieldValuePair {
    binding: String,
    value: PairValue,
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
                    self.save_field_value(
                        field.binding.clone(),
                        PairValue {
                            is_preset: false,
                            value: default_value.clone(),
                        },
                    )?;
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
    use wasm_bindgen_test::wasm_bindgen_test;

    const YAML: &str = r#"
gui:
  name: Fixed limit
  description: Fixed limit order strategy
  short-description: Buy WETH with USDC on Base.
  deployments:
    some-deployment:
      name: Buy WETH with USDC on Base.
      description: Buy WETH with USDC for fixed price on Base network.
      short-description: Buy WETH with USDC on Base.
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
            - "10"
            - "100"
            - "1000"
            - "10000"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0x1234567890abcdef1234567890abcdef12345678"
            - name: Preset 2
              value: "false"
            - name: Preset 3
              value: "some-string"
          default: some-default-value
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          presets:
            - value: "99.2"
            - value: "582.1"
            - value: "648.239"
          show-custom-field: true
    other-deployment:
      name: Test test
      description: Test test test
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
networks:
    some-network:
        rpc: http://localhost:8085/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH
subgraphs:
    some-sg: https://www.some-sg.com
metaboards:
    test: https://metaboard.com
deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: Token 1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2
scenarios:
    some-scenario:
        deployer: some-deployer
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300
orders:
    some-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token2
          vault-id: 1
      deployer: some-deployer
      orderbook: some-orderbook
deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    other-deployment:
        scenario: some-scenario.sub-scenario
        order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
    "#;

    #[wasm_bindgen_test]
    async fn test_get_field_value() {
        let mut gui = DotrainOrderGui::new();

        gui.choose_deployment(YAML.to_string(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        gui.save_field_value(
            "binding-1".to_string(),
            PairValue {
                is_preset: false,
                value: "some-default-value".to_string(),
            },
        )
        .unwrap();
        gui.save_field_value(
            "binding-2".to_string(),
            PairValue {
                is_preset: true,
                value: "0".to_string(),
            },
        )
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
    async fn test_get_all_field_values() {
        let mut gui = DotrainOrderGui::new();

        gui.choose_deployment(YAML.to_string(), "some-deployment".to_string(), None)
            .await
            .unwrap();

        gui.save_field_value(
            "binding-1".to_string(),
            PairValue {
                is_preset: false,
                value: "some-default-value".to_string(),
            },
        )
        .unwrap();
        gui.save_field_value(
            "binding-2".to_string(),
            PairValue {
                is_preset: true,
                value: "0".to_string(),
            },
        )
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
}
