pub mod prelude {
    pub use serde_wasm_bindgen::{from_value, to_value};
    pub use tsify::Tsify;
    pub use wasm_bindgen::{
        convert::*,
        describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR},
        prelude::*,
        JsValue, UnwrapThrowExt,
    };
}

#[macro_export]
macro_rules! impl_main_wasm_traits {
    ($type_name:path) => {{
        use $crate::wasm_traits::prelude::*;

        impl WasmDescribe for $type_name {
            #[inline]
            fn describe() {
                <Self as Tsify>::JsType::describe()
            }
        }
        impl IntoWasmAbi for $type_name {
            type Abi = <<Self as Tsify>::JsType as IntoWasmAbi>::Abi;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                let mut err = "".to_string();
                self.into_js()
                    .inspect_err(|e| err.push_str(&e.to_string()))
                    .expect_throw(&err)
                    .into_abi()
            }
        }
        impl OptionIntoWasmAbi for $type_name {
            #[inline]
            fn none() -> Self::Abi {
                <<Self as Tsify>::JsType as OptionIntoWasmAbi>::none()
            }
        }
        impl FromWasmAbi for $type_name {
            type Abi = <<Self as Tsify>::JsType as FromWasmAbi>::Abi;

            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                let mut err = "".to_string();
                Self::from_js(<Self as Tsify>::JsType::from_abi(js))
                    .inspect_err(|e| err.push_str(&e.to_string()))
                    .expect_throw(&err)
            }
        }
        impl OptionFromWasmAbi for $type_name {
            #[inline]
            fn is_none(js: &Self::Abi) -> bool {
                <<Self as Tsify>::JsType as OptionFromWasmAbi>::is_none(js)
            }
        }
    }};
}

#[macro_export]
macro_rules! impl_complementary_wasm_traits {
    ($type_name:path) => {{
        use $crate::wasm_traits::prelude::*;

        impl RefFromWasmAbi for $type_name {
            type Abi = <JsValue as RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name>::from_abi(js))
            }
        }
        impl LongRefFromWasmAbi for $type_name {
            type Abi = <JsValue as RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name>::from_abi(js))
            }
        }
        impl VectorIntoWasmAbi for $type_name {
            type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
            fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
                js_value_vector_into_abi(vector)
            }
        }
        impl VectorFromWasmAbi for $type_name {
            type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
            unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
                js_value_vector_from_abi(js)
            }
        }
        impl WasmDescribeVector for $type_name {
            fn describe_vector() {
                inform(VECTOR);
                <$type_name>::describe();
            }
        }
        impl From<$type_name> for JsValue {
            fn from(value: $type_name) -> Self {
                let mut err = "".to_string();
                to_value(&value)
                    .inspect_err(|e| err.push_str(&e.to_string()))
                    .expect_throw(&err)
            }
        }
        impl TryFromJsValue for $type_name {
            type Error = serde_wasm_bindgen::Error;
            fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
                from_value(value)
            }
        }
    }};
}

#[macro_export]
macro_rules! impl_all_wasm_traits {
    ($type_name:path) => {
        $crate::impl_main_wasm_traits!($type_name);
        $crate::impl_complementary_wasm_traits!($type_name);
    };
}
