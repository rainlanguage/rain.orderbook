#[macro_use]
extern crate rain_orderbook_macros;
struct TestStruct;
impl TestStruct {
    pub async fn some_static_method((arg, e): (String, u8)) -> Result<String, Error> {
        Ok(String::new())
    }
    pub async fn some_skip_fn() -> Result<String, Error> {
        Ok(String::new())
    }
    pub async fn some_self_method(&self, arg: String) -> Result<TestStruct, Error> {
        Ok(TestStruct)
    }
    pub fn returns_num_array(&mut self) -> Result<Vec<u8>, Error> {
        Ok(::alloc::vec::Vec::new())
    }
}
#[wasm_bindgen]
impl TestStruct {
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "someStaticMethod")]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<string>")]
    pub async fn some_static_method__wasm_export(
        (arg, e): (String, u8),
    ) -> WasmEncodedResult<String> {
        Self::some_static_method((arg, e)).await.into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "someSelfMethod")]
    #[wasm_bindgen(some_other_wbg_attrs)]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<TestStruct>")]
    pub async fn some_self_method__wasm_export(
        &self,
        arg: String,
    ) -> WasmEncodedResult<TestStruct> {
        self.some_self_method(arg).await.into()
    }
    #[allow(non_snake_case)]
    #[wasm_bindgen(js_name = "returnsNumArray")]
    #[wasm_bindgen(unchecked_return_type = "WasmEncodedResult<number[]>")]
    pub fn returns_num_array__wasm_export(&mut self) -> WasmEncodedResult<Vec<u8>> {
        self.returns_num_array().into()
    }
}
