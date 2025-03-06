#[macro_use]
extern crate rain_orderbook_macros;
struct TestStruct;
impl TestStruct {
    #[allow(non_snake_case)]
    pub async fn get_deployment_keys(dotrain: String) -> Result<String, Error> {
        Ok(String::new())
    }
    #[allow(non_snake_case)]
    pub async fn choose_deployment(
        &mut self,
        dotrain: String,
        deployment_name: String,
    ) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
}
#[wasm_bindgen]
impl TestStruct {
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "getDeploymentKeys", unchecked_return_type = "string")]
    pub async fn get_deployment_keys__wasm_export(
        dotrain: String,
    ) -> WasmEncodedResult<String> {
        Self::get_deployment_keys(dotrain).await.into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "chooseDeployment")]
    pub async fn choose_deployment__wasm_export(
        &mut self,
        dotrain: String,
        deployment_name: String,
    ) -> WasmEncodedResult<TestStruct> {
        self.choose_deployment(dotrain, deployment_name).await.into()
    }
}
