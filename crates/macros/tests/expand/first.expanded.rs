#[macro_use]
extern crate rain_orderbook_macros;
struct TestStruct;
impl TestStruct {
    pub async fn get_deployment_keys(dotrain: String) -> Result<String, Error> {
        Ok(String::new())
    }
    pub async fn some_skip_fn() -> Result<String, Error> {
        Ok(String::new())
    }
    pub async fn choose_deployment(
        &mut self,
        dotrain: String,
        deployment_name: String,
    ) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
    pub fn return_num_array(&mut self) -> Result<Vec<u8>, Error> {
        Ok(::alloc::vec::Vec::new())
    }
}
#[wasm_bindgen]
impl TestStruct {
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "getDeploymentKeys")]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<string>")]
    pub async fn get_deployment_keys__wasm_export(
        dotrain: String,
    ) -> WasmEncodedResult<String> {
        Self::get_deployment_keys(dotrain).await.into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "chooseDeployment")]
    #[wasm_bindgen(optional)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<TestStruct>")]
    pub async fn choose_deployment__wasm_export(
        &mut self,
        dotrain: String,
        deployment_name: String,
    ) -> WasmEncodedResult<TestStruct> {
        self.choose_deployment(dotrain, deployment_name).await.into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "returnNumArray")]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<number[]>")]
    pub fn return_num_array__wasm_export(&mut self) -> WasmEncodedResult<Vec<u8>> {
        self.return_num_array().into()
    }
}
