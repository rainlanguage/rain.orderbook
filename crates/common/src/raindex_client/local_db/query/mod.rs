pub mod clear_tables;
pub mod create_tables;
pub mod fetch_erc20_tokens_by_addresses;
pub mod fetch_last_synced_block;
pub mod fetch_order_trades;
pub mod fetch_order_trades_count;
pub mod fetch_orders;
pub mod fetch_store_addresses;
pub mod fetch_tables;
pub mod fetch_vault;
pub mod fetch_vault_balance_changes;
pub mod fetch_vaults;
pub mod update_last_synced_block;

use super::*;
use wasm_bindgen_utils::prelude::wasm_bindgen_futures::JsFuture;

pub struct LocalDbQuery;

#[derive(Error, Debug)]
pub enum LocalDbQueryError {
    #[error("JavaScript callback invocation failed: {0}")]
    CallbackError(String),

    #[error("Promise resolution failed: {0}")]
    PromiseError(String),

    #[error("JSON deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Database operation failed: {message}")]
    DatabaseError { message: String },

    #[error("Invalid response format from database")]
    InvalidResponse,
}

impl LocalDbQuery {
    async fn execute_query_raw(
        callback: &js_sys::Function,
        sql: &str,
    ) -> Result<String, LocalDbQueryError> {
        let result = callback
            .call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(sql),
            )
            .map_err(|e| LocalDbQueryError::CallbackError(format!("{:?}", e)))?;

        let promise = js_sys::Promise::resolve(&result);
        let future = JsFuture::from(promise);

        let js_result = future
            .await
            .map_err(|e| LocalDbQueryError::PromiseError(format!("{:?}", e)))?;

        let wasm_result: WasmEncodedResult<String> = serde_wasm_bindgen::from_value(js_result)
            .map_err(|_| LocalDbQueryError::InvalidResponse)?;

        match wasm_result {
            WasmEncodedResult::Success { value, .. } => Ok(value),
            WasmEncodedResult::Err { error, .. } => Err(LocalDbQueryError::DatabaseError {
                message: error.readable_msg,
            }),
        }
    }

    pub async fn execute_query_json<T>(
        callback: &js_sys::Function,
        sql: &str,
    ) -> Result<T, LocalDbQueryError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let value = Self::execute_query_raw(callback, sql).await?;

        serde_json::from_str(&value).map_err(LocalDbQueryError::JsonError)
    }

    pub async fn execute_query_text(
        callback: &js_sys::Function,
        sql: &str,
    ) -> Result<String, LocalDbQueryError> {
        Self::execute_query_raw(callback, sql).await
    }
}

pub mod serde_address {
    use alloy::{hex::encode_prefixed, primitives::Address};
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &Address, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode_prefixed(value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Address, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Address::from_str(&s).map_err(Error::custom)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Wrapper {
            #[serde(with = "super")]
            addr: Address,
        }

        #[test]
        fn serialize_address_to_prefixed_hex() {
            let wrapper = Wrapper {
                addr: Address::from_str("0x123400000000000000000000000000000000abcd").unwrap(),
            };
            let json = serde_json::to_string(&wrapper).expect("serialize succeeds");
            assert!(json.contains(&encode_prefixed(wrapper.addr)));
        }

        #[test]
        fn deserialize_prefixed_hex_into_address() {
            let encoded = encode_prefixed(
                Address::from_str("0x0000000000000000000000000000000000000001").unwrap(),
            );
            let json = format!(r#"{{"addr":"{}"}}"#, encoded);
            let wrapper: Wrapper = serde_json::from_str(&json).expect("deserialize succeeds");
            assert_eq!(
                wrapper.addr,
                Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
            );
        }

        #[test]
        fn deserialize_rejects_invalid_hex() {
            let json = r#"{"addr":"0xzz"}"#;
            let err = serde_json::from_str::<Wrapper>(json).expect_err("must fail");
            assert!(
                err.to_string().to_lowercase().contains("invalid"),
                "unexpected error: {err}"
            );
        }
    }
}

pub mod serde_bytes {
    use alloy::{hex::encode_prefixed, primitives::Bytes};
    use serde::{de::Error as DeError, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode_prefixed(value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Bytes::from_str(&s).map_err(DeError::custom)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Wrapper {
            #[serde(with = "super")]
            data: Bytes,
        }

        #[test]
        fn serialize_bytes_to_prefixed_hex() {
            let wrapper = Wrapper {
                data: Bytes::from_str("0xdeadbeef").unwrap(),
            };
            let json = serde_json::to_string(&wrapper).expect("serialize succeeds");
            assert_eq!(
                json,
                format!(r#"{{"data":"{}"}}"#, encode_prefixed(&wrapper.data))
            );
        }

        #[test]
        fn deserialize_prefixed_hex_into_bytes() {
            let json = r#"{"data":"0x0102"}"#;
            let wrapper: Wrapper = serde_json::from_str(json).expect("deserialize succeeds");
            assert_eq!(wrapper.data, Bytes::from_str("0x0102").unwrap());
        }

        #[test]
        fn deserialize_rejects_invalid_hex() {
            let json = r#"{"data":"0x1g"}"#;
            let err = serde_json::from_str::<Wrapper>(json).expect_err("must fail");
            assert!(
                err.to_string().to_lowercase().contains("invalid"),
                "unexpected error: {err}"
            );
        }
    }
}

pub mod serde_option_bytes {
    use alloy::{hex::encode_prefixed, primitives::Bytes};
    use serde::{de::Error as DeError, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &Option<Bytes>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => serializer.serialize_some(&encode_prefixed(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Bytes>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let maybe_str = Option::<String>::deserialize(deserializer)?;
        maybe_str
            .map(|s| Bytes::from_str(&s).map_err(DeError::custom))
            .transpose()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Wrapper {
            #[serde(with = "super")]
            data: Option<Bytes>,
        }

        #[test]
        fn serialize_some_bytes_to_prefixed_hex() {
            let wrapper = Wrapper {
                data: Some(Bytes::from_str("0xdeadbeef").unwrap()),
            };
            let json = serde_json::to_string(&wrapper).expect("serialize succeeds");
            assert_eq!(
                json,
                format!(
                    r#"{{"data":"{}"}}"#,
                    encode_prefixed(wrapper.data.as_ref().unwrap())
                )
            );
        }

        #[test]
        fn serialize_none_bytes() {
            let wrapper = Wrapper { data: None };
            let json = serde_json::to_string(&wrapper).expect("serialize succeeds");
            assert_eq!(json, r#"{"data":null}"#);
        }

        #[test]
        fn deserialize_prefixed_hex_into_some_bytes() {
            let json = r#"{"data":"0x0102"}"#;
            let wrapper: Wrapper = serde_json::from_str(json).expect("deserialize succeeds");
            assert_eq!(wrapper.data, Some(Bytes::from_str("0x0102").unwrap()));
        }

        #[test]
        fn deserialize_null_into_none_bytes() {
            let json = r#"{"data":null}"#;
            let wrapper: Wrapper = serde_json::from_str(json).expect("deserialize succeeds");
            assert!(wrapper.data.is_none());
        }

        #[test]
        fn deserialize_rejects_invalid_hex() {
            let json = r#"{"data":"0x1g"}"#;
            let err = serde_json::from_str::<Wrapper>(json).expect_err("must fail");
            assert!(
                err.to_string().to_lowercase().contains("invalid"),
                "unexpected error: {err}"
            );
        }
    }
}

pub mod serde_float {
    use rain_math_float::Float;
    use serde::{
        de::Error as DeError, ser::Error as SerError, Deserialize, Deserializer, Serializer,
    };

    pub fn serialize<S>(value: &Float, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.format().map_err(SerError::custom)?)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Float, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Float::parse(s).map_err(DeError::custom)
    }

    #[cfg(test)]
    mod tests {
        use super::Float;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct Wrapper {
            #[serde(with = "super")]
            value: Float,
        }

        #[test]
        fn serialize_float_uses_formatted_string() {
            let float = Float::parse("42.5".into()).unwrap();
            let wrapper = Wrapper { value: float };
            let json = serde_json::to_string(&wrapper).expect("serialize succeeds");
            assert!(json.contains("\"42.5\""));
        }

        #[test]
        fn deserialize_parses_float_string() {
            let json = r#"{"value":"1.25"}"#;
            let wrapper: Wrapper = serde_json::from_str(json).expect("deserialize succeeds");
            assert_eq!(wrapper.value.format().unwrap(), "1.25");
        }

        #[test]
        fn deserialize_invalid_float_errors() {
            let json = r#"{"value":"not-a-number"}"#;
            let err = serde_json::from_str::<Wrapper>(json).expect_err("must fail");
            assert!(
                err.to_string().contains("Decimal Float error selector"),
                "unexpected error: {err}"
            );
        }
    }
}

pub mod serde_option_float {
    use rain_math_float::Float;
    use serde::{
        de::Error as DeError, ser::Error as SerError, Deserialize, Deserializer, Serializer,
    };

    pub fn serialize<S>(value: &Option<Float>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(float) => {
                let formatted = float.format().map_err(SerError::custom)?;
                serializer.serialize_some(&formatted)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Float>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let maybe_str = Option::<String>::deserialize(deserializer)?;
        maybe_str
            .map(|s| Float::parse(s).map_err(DeError::custom))
            .transpose()
    }

    #[cfg(test)]
    mod tests {
        use super::Float;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct Wrapper {
            #[serde(with = "super")]
            value: Option<Float>,
        }

        #[test]
        fn serialize_some_float() {
            let float = Float::parse("0.75".into()).unwrap();
            let wrapper = Wrapper { value: Some(float) };
            let json = serde_json::to_string(&wrapper).expect("serialize succeeds");
            assert!(json.contains("\"0.75\""));
        }

        #[test]
        fn serialize_none_float() {
            let wrapper = Wrapper { value: None };
            let json = serde_json::to_string(&wrapper).expect("serialize succeeds");
            assert_eq!(json, r#"{"value":null}"#);
        }

        #[test]
        fn deserialize_some_float() {
            let json = r#"{"value":"123.0001"}"#;
            let wrapper: Wrapper = serde_json::from_str(json).expect("deserialize succeeds");
            assert_eq!(wrapper.value.unwrap().format().unwrap(), "123.0001");
        }

        #[test]
        fn deserialize_none_float() {
            let json = r#"{"value":null}"#;
            let wrapper: Wrapper = serde_json::from_str(json).expect("deserialize succeeds");
            assert!(wrapper.value.is_none());
        }
    }
}

pub mod serde_b256 {
    use alloy::{
        hex::encode_prefixed,
        primitives::{B256, U256},
    };
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode_prefixed(B256::from(*value)))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if !s.starts_with("0x") {
            return Err(Error::custom("expected 0x-prefixed hex string"));
        }

        let b256 =
            B256::from_str(&s).map_err(|e| Error::custom(format!("invalid hex string: {e}")))?;

        Ok(U256::from_be_slice(b256.as_slice()))
    }

    #[cfg(test)]
    mod tests {
        use alloy::{
            hex::encode_prefixed,
            primitives::{B256, U256},
        };
        use serde::{Deserialize, Serialize};
        use std::str::FromStr;

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Wrapper {
            #[serde(with = "super")]
            value: U256,
        }

        #[test]
        fn serialize_u256_to_prefixed_hex() {
            let wrapper = Wrapper {
                value: U256::from_str(
                    "0x01020000000000000000000000000000000000000000000000000000000000ff",
                )
                .unwrap(),
            };
            let json = serde_json::to_string(&wrapper).expect("serialization succeeds");
            assert_eq!(
                json,
                format!(
                    r#"{{"value":"{}"}}"#,
                    encode_prefixed(B256::from(wrapper.value))
                )
            );
        }

        #[test]
        fn deserialize_prefixed_hex_into_u256() {
            let encoded = encode_prefixed(B256::from(U256::from(42_u32)));
            let json = format!(r#"{{"value":"{}"}}"#, encoded);
            let wrapper: Wrapper = serde_json::from_str(&json).expect("deserialization succeeds");
            assert_eq!(wrapper.value, U256::from(42_u32));
        }

        #[test]
        fn deserialize_rejects_missing_prefix() {
            let json = r#"{"value":"1234"}"#;
            let err = serde_json::from_str::<Wrapper>(json).expect_err("must fail");
            assert!(
                err.to_string().contains("0x-prefixed"),
                "unexpected error: {err}"
            );
        }
    }
}

#[cfg(test)]
pub mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use js_sys::Function;
        use serde::{Deserialize, Serialize};
        use wasm_bindgen_test::*;

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TestData {
            id: u32,
            name: String,
        }

        fn create_error_callback(readable_msg: &str) -> Function {
            let error_result = WasmEncodedResult::Err::<String> {
                value: None,
                error: WasmEncodedError {
                    msg: "DatabaseError".to_string(),
                    readable_msg: readable_msg.to_string(),
                },
            };
            let js_value = serde_wasm_bindgen::to_value(&error_result).unwrap();

            Function::new_no_args(&format!(
                "return {}",
                js_sys::JSON::stringify(&js_value)
                    .unwrap()
                    .as_string()
                    .unwrap()
            ))
        }

        fn create_invalid_callback() -> Function {
            Function::new_no_args("return 42")
        }

        fn create_callback_that_throws() -> Function {
            Function::new_no_args("throw new Error('Callback error')")
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_success_case() {
            let test_data = vec![
                TestData {
                    id: 1,
                    name: "Alice".to_string(),
                },
                TestData {
                    id: 2,
                    name: "Bob".to_string(),
                },
            ];
            let json_data = serde_json::to_string(&test_data).unwrap();
            let callback = super::create_success_callback(&json_data);

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT * FROM users").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].name, "Alice");
            assert_eq!(data[1].name, "Bob");
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_empty_success() {
            let callback = super::create_success_callback("[]");

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT * FROM empty_table").await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_database_error() {
            let callback = create_error_callback("no such table: users");

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::DatabaseError { message } => {
                    assert_eq!(message, "no such table: users");
                }
                _ => panic!("Expected DatabaseError"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_invalid_json() {
            let callback = super::create_success_callback("{ invalid json }");

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::JsonError(_) => {}
                _ => panic!("Expected JsonError"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_invalid_response_format() {
            let callback = create_invalid_callback();

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::InvalidResponse => {}
                _ => panic!("Expected InvalidResponse"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_callback_throws() {
            let callback = create_callback_that_throws();

            let result: Result<Vec<TestData>, LocalDbQueryError> =
                LocalDbQuery::execute_query_json(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::CallbackError(_) => {}
                _ => panic!("Expected CallbackError"),
            }
        }

        fn create_rejecting_promise_callback() -> Function {
            Function::new_no_args("return Promise.reject(new Error('Promise failed'))")
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_success() {
            let callback = super::create_success_callback("hello world");

            let result = LocalDbQuery::execute_query_text(&callback, "SELECT 'hello world'").await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "hello world".to_string());
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_empty_success() {
            let callback = super::create_success_callback("");

            let result = LocalDbQuery::execute_query_text(&callback, "SELECT ''").await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "");
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_database_error() {
            let callback = create_error_callback("no such table: users");

            let result = LocalDbQuery::execute_query_text(&callback, "SELECT * FROM users").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::DatabaseError { message } => {
                    assert_eq!(message, "no such table: users");
                }
                _ => panic!("Expected DatabaseError"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_invalid_response_format() {
            let callback = create_invalid_callback();

            let result = LocalDbQuery::execute_query_text(&callback, "SELECT 1").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::InvalidResponse => {}
                _ => panic!("Expected InvalidResponse"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_callback_throws() {
            let callback = create_callback_that_throws();

            let result = LocalDbQuery::execute_query_text(&callback, "SELECT 1").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::CallbackError(_) => {}
                _ => panic!("Expected CallbackError"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_promise_rejection() {
            let callback = create_rejecting_promise_callback();

            let result = LocalDbQuery::execute_query_text(&callback, "SELECT 1").await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::PromiseError(_) => {}
                _ => panic!("Expected PromiseError"),
            }
        }

        #[wasm_bindgen_test]
        async fn test_execute_query_text_passes_sql_to_callback() {
            use std::cell::RefCell;
            use std::rc::Rc;

            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = super::create_sql_capturing_callback("ok", captured_sql.clone());

            let sql = "SELECT name FROM users WHERE id = 42";
            let result = LocalDbQuery::execute_query_text(&callback, sql).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "ok".to_string());
            assert_eq!(captured_sql.borrow().as_str(), sql);
        }
    }

    #[cfg(target_family = "wasm")]
    pub fn create_success_callback(json_data: &str) -> js_sys::Function {
        let success_result = WasmEncodedResult::Success::<String> {
            value: json_data.to_string(),
            error: None,
        };
        let js_value = serde_wasm_bindgen::to_value(&success_result).unwrap();

        js_sys::Function::new_no_args(&format!(
            "return {}",
            js_sys::JSON::stringify(&js_value)
                .unwrap()
                .as_string()
                .unwrap()
        ))
    }

    #[cfg(target_family = "wasm")]
    pub fn create_sql_capturing_callback(
        json_data: &str,
        captured_sql: std::rc::Rc<std::cell::RefCell<String>>,
    ) -> js_sys::Function {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let success_result = WasmEncodedResult::Success::<String> {
            value: json_data.to_string(),
            error: None,
        };
        let js_value = serde_wasm_bindgen::to_value(&success_result).unwrap();

        let result_json = js_sys::JSON::stringify(&js_value)
            .unwrap()
            .as_string()
            .unwrap();

        let callback = Closure::wrap(Box::new(move |sql: String| -> JsValue {
            *captured_sql.borrow_mut() = sql;
            js_sys::JSON::parse(&result_json).unwrap()
        }) as Box<dyn Fn(String) -> JsValue>);

        callback.into_js_value().dyn_into().unwrap()
    }
}
