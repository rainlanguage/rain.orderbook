use super::*;
use rain_orderbook_common::raindex_order_builder::state_management as inner_sm;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AllGuiConfig {
    pub field_definitions_without_defaults: Vec<GuiFieldDefinitionCfg>,
    pub field_definitions_with_defaults: Vec<GuiFieldDefinitionCfg>,
    pub deposits: Vec<rain_orderbook_app_settings::gui::GuiDepositCfg>,
    pub order_inputs: Vec<rain_orderbook_app_settings::order::OrderIOCfg>,
    pub order_outputs: Vec<rain_orderbook_app_settings::order::OrderIOCfg>,
}
impl_wasm_traits!(AllGuiConfig);

impl RaindexOrderBuilder {
    pub fn execute_state_update_callback(&self) -> Result<(), RaindexOrderBuilderWasmError> {
        if let Some(callback) = &self.state_update_callback {
            let serialized = self.inner.serialize_state()?;
            let js_value = JsValue::from_str(&serialized);
            callback.call1(&JsValue::NULL, &js_value).map_err(|e| {
                RaindexOrderBuilderWasmError::JsError(
                    e.as_string().unwrap_or("callback error".to_string()),
                )
            })?;
        }
        Ok(())
    }
}

#[wasm_export]
impl RaindexOrderBuilder {
    #[wasm_export(
        js_name = "serializeState",
        unchecked_return_type = "string",
        return_description = "Base64-encoded compressed serialized state"
    )]
    pub fn serialize_state(&self) -> Result<String, RaindexOrderBuilderWasmError> {
        Ok(self.inner.serialize_state()?)
    }

    #[wasm_export(
        js_name = "newFromState",
        preserve_js_class,
        return_description = "Restored GUI instance"
    )]
    pub async fn new_from_state(
        #[wasm_export(param_description = "Complete dotrain YAML content")] dotrain: String,
        #[wasm_export(param_description = "Optional additional YAML settings")] settings: Option<
            Vec<String>,
        >,
        #[wasm_export(param_description = "Serialized state string from serializeState")]
        serialized: String,
        #[wasm_export(param_description = "Optional state update callback function")]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<RaindexOrderBuilder, RaindexOrderBuilderWasmError> {
        let inner = RaindexOrderBuilderInner::new_from_state(dotrain, settings, serialized).await?;
        Ok(RaindexOrderBuilder {
            inner,
            state_update_callback,
        })
    }

    #[wasm_export(
        js_name = "getAllGuiConfig",
        unchecked_return_type = "AllGuiConfig",
        return_description = "Complete GUI configuration for all fields, deposits, and I/O"
    )]
    pub fn get_all_gui_config(
        &self,
    ) -> Result<inner_sm::AllGuiConfig, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_all_gui_config()?)
    }

    #[wasm_export(
        js_name = "computeStateHash",
        unchecked_return_type = "string",
        return_description = "Base64-encoded SHA256 hash of the dotrain content"
    )]
    pub async fn compute_state_hash(
        #[wasm_export(param_description = "Complete dotrain YAML content")] dotrain: String,
        #[wasm_export(param_description = "Optional additional YAML settings")] settings: Option<
            Vec<String>,
        >,
    ) -> Result<String, RaindexOrderBuilderWasmError> {
        let dotrain_order =
            rain_orderbook_common::dotrain_order::DotrainOrder::create(dotrain, settings)
                .await
                .map_err(RaindexOrderBuilderError::from)?;
        Ok(RaindexOrderBuilderInner::compute_state_hash(
            &dotrain_order,
        )?)
    }

    #[wasm_export(
        js_name = "generateDotrainGuiStateInstanceV1",
        unchecked_return_type = "DotrainGuiStateV1",
        return_description = "GUI state instance for metadata embedding"
    )]
    pub fn generate_dotrain_gui_state_instance_v1(
        &self,
    ) -> Result<
        rain_metadata::types::dotrain::gui_state_v1::DotrainGuiStateV1,
        RaindexOrderBuilderWasmError,
    > {
        Ok(self.inner.generate_dotrain_gui_state_instance_v1()?)
    }
}
