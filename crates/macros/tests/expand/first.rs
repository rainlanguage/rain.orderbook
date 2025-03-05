#[macro_use]
extern crate rain_orderbook_macros;

struct TestStruct;

#[impl_wasm_exports]
impl TestStruct {
    #[wasm_export(
        js_name = "getDeploymentKeys",
        unchecked_return_type = "string"
    )]
    pub async fn get_deployment_keys(dotrain: String) -> Result<String, Error> {
        Ok(String::new())
    }

    #[wasm_export(js_name = "chooseDeployment")]
    pub async fn choose_deployment(
        &mut self,
        dotrain: String,
        deployment_name: String,
    ) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
}