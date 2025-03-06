#[macro_use]
extern crate rain_orderbook_macros;

struct TestStruct;

#[impl_wasm_exports]
impl TestStruct {
    #[wasm_export(js_name = "someStaticMethod", unchecked_return_type = "string")]
    pub async fn some_static_method(arg: String) -> Result<String, Error> {
        Ok(String::new())
    }

    #[wasm_export(skip)]
    pub async fn some_skip_fn() -> Result<String, Error> {
        Ok(String::new())
    }

    #[wasm_export(js_name = "someSelfMethod", some_other_wbg_attrs)]
    pub async fn some_self_method(&self, arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }

    #[wasm_export(unchecked_return_type = "number[]", js_name = "returnsNumArray")]
    pub fn returns_num_array(&mut self) -> Result<Vec<u8>, Error> {
        Ok(vec![])
    }
}
