use rain_orderbook_app_settings::yaml::{dotrain::DotrainYaml, YamlParsable};
use rain_orderbook_common::dotrain_order::DotrainOrder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use wasm_function_macro::{impl_wasm_exports, wasm_export};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct WasmEncodedError {
    msg: String,
    readable_msg: String,
}
impl_wasm_traits!(WasmEncodedError);

impl From<TestError> for WasmEncodedError {
    fn from(err: TestError) -> Self {
        WasmEncodedError {
            msg: err.to_string(),
            readable_msg: err.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WasmEncodedResult<T> {
    data: Option<T>,
    error: Option<WasmEncodedError>,
}

impl<T> WasmEncodedResult<T> {
    pub fn success(data: T) -> Self {
        WasmEncodedResult {
            data: Some(data),
            error: None,
        }
    }

    pub fn error(err: WasmEncodedError) -> Self {
        WasmEncodedResult {
            data: None,
            error: Some(err),
        }
    }
}

impl<T> From<Result<T, TestError>> for WasmEncodedResult<T> {
    fn from(result: Result<T, TestError>) -> Self {
        match result {
            Ok(data) => WasmEncodedResult {
                data: Some(data),
                error: None,
            },
            Err(err) => WasmEncodedResult {
                data: None,
                error: Some(WasmEncodedError {
                    msg: err.to_string(),
                    readable_msg: err.to_readable_msg(),
                }),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TestReturnType {
    field: String,
}
impl_wasm_traits!(TestReturnType);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct HashMapReturnType(pub HashMap<String, u64>);
impl_wasm_traits!(HashMapReturnType);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct VecReturnType(pub Vec<u64>);
impl_wasm_traits!(VecReturnType);

#[derive(Error, Debug)]
pub enum TestError {
    #[error("Test error")]
    TestError,
    #[error("JavaScript error: {0}")]
    JsError(String),
}
impl TestError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            TestError::TestError => "An unexpected error occurred. Please try again.".to_string(),
            TestError::JsError(msg) => format!("Something went wrong: {}", msg),
        }
    }
}
impl From<TestError> for JsValue {
    fn from(value: TestError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct TestStruct {
    field: String,
}

#[wasm_bindgen]
impl TestStruct {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TestStruct {
        TestStruct {
            field: "".to_string(),
        }
    }
}

#[impl_wasm_exports]
impl TestStruct {
    #[wasm_export(js_name = "updateField")]
    pub fn update_field(&mut self, value: String) -> Result<(), TestError> {
        self.field = value;
        Ok(())
    }

    // #[wasm_export(js_name = "saveDotrain")]
    // pub async fn new_with_dotrain_result(&mut self, value: String) -> Result<(), TestError> {
    //     let dotrain_order = DotrainOrder::new(value, None).await?;
    //     let dotrain =
    //         DotrainYaml::get_yaml_string(dotrain_order.dotrain_yaml().documents[0].clone())?;
    //     self.field = dotrain;
    //     Ok(())
    // }

    #[wasm_export(js_name = "simpleFunction", unchecked_return_type = "string")]
    pub fn simple_function() -> Result<String, TestError> {
        Ok("Hello, world!".to_string())
    }

    #[wasm_export(js_name = "errFunction", unchecked_return_type = "string")]
    pub fn err_function() -> Result<String, TestError> {
        Err(TestError::JsError("some error".to_string()))
    }

    #[wasm_export(js_name = "simpleFunctionWithSelf", unchecked_return_type = "string")]
    pub fn simple_function_with_self(&self) -> Result<String, TestError> {
        Ok(format!("Hello, {}!", self.field))
    }

    #[wasm_export(js_name = "errFunctionWithSelf", unchecked_return_type = "string")]
    pub fn err_function_with_self(&self) -> Result<String, TestError> {
        Err(TestError::TestError)
    }

    #[wasm_export(
        js_name = "simpleFunctionWithReturnType",
        unchecked_return_type = "TestReturnType"
    )]
    pub fn simple_function_with_return_type() -> Result<TestReturnType, TestError> {
        Ok(TestReturnType {
            field: "Hello, world!".to_string(),
        })
    }

    #[wasm_export(
        js_name = "simpleFunctionWithReturnTypeWithSelf",
        unchecked_return_type = "TestReturnType"
    )]
    pub fn simple_function_with_return_type_with_self(&self) -> Result<TestReturnType, TestError> {
        Ok(TestReturnType {
            field: format!("Hello, {}!", self.field),
        })
    }

    #[wasm_export(js_name = "asyncFunction", unchecked_return_type = "number")]
    pub async fn async_function() -> Result<u64, TestError> {
        Ok(123)
    }

    #[wasm_export(js_name = "asyncFunctionWithSelf", unchecked_return_type = "number")]
    pub async fn async_function_with_self(self) -> Result<u64, TestError> {
        Ok(234)
    }

    #[wasm_export(js_name = "returnVec", unchecked_return_type = "VecReturnType")]
    pub fn return_vec() -> Result<VecReturnType, TestError> {
        Ok(VecReturnType(vec![1, 2, 3]))
    }

    #[wasm_export(js_name = "returnHashmap", unchecked_return_type = "HashMapReturnType")]
    pub fn return_hashmap() -> Result<HashMapReturnType, TestError> {
        Ok(HashMapReturnType(HashMap::from([("key".to_string(), 123)])))
    }

    #[wasm_export(js_name = "returnOption", unchecked_return_type = "number | undefined")]
    pub fn return_option() -> Result<Option<u64>, TestError> {
        Ok(Some(123))
    }

    #[wasm_export(
        js_name = "returnOptionNone",
        unchecked_return_type = "number | undefined"
    )]
    pub fn return_option_none() -> Result<Option<u64>, TestError> {
        Ok(None)
    }

    fn private_function() -> String {
        "Hello, world!".to_string()
    }
}

impl_wasm_traits!(WasmEncodedResult<String>);
impl_wasm_traits!(WasmEncodedResult<u64>);
impl_wasm_traits!(WasmEncodedResult<TestReturnType>);
impl_wasm_traits!(WasmEncodedResult<VecReturnType>);
impl_wasm_traits!(WasmEncodedResult<HashMapReturnType>);
impl_wasm_traits!(WasmEncodedResult<Option<u64>>);
impl_wasm_traits!(WasmEncodedResult<()>);
