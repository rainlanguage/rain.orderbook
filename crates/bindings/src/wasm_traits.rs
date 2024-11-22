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
    ($type_name:path) => {
        impl $crate::wasm_traits::prelude::WasmDescribe for $type_name {
            #[inline]
            fn describe() {
                <Self as $crate::wasm_traits::prelude::Tsify>::JsType::describe()
            }
        }
        impl $crate::wasm_traits::prelude::IntoWasmAbi for $type_name {
            type Abi = <<Self as $crate::wasm_traits::prelude::Tsify>::JsType as $crate::wasm_traits::prelude::IntoWasmAbi>::Abi;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                let mut err = "".to_string();
                let result = $crate::wasm_traits::prelude::Tsify::into_js(&self);
                $crate::wasm_traits::prelude::UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err).into_abi()
            }
        }
        impl $crate::wasm_traits::prelude::OptionIntoWasmAbi for $type_name {
            #[inline]
            fn none() -> Self::Abi {
                <<Self as $crate::wasm_traits::prelude::Tsify>::JsType as $crate::wasm_traits::prelude::OptionIntoWasmAbi>::none()
            }
        }
        impl $crate::wasm_traits::prelude::FromWasmAbi for $type_name {
            type Abi = <<Self as $crate::wasm_traits::prelude::Tsify>::JsType as $crate::wasm_traits::prelude::FromWasmAbi>::Abi;

            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                let mut err = "".to_string();
                let result = <Self as $crate::wasm_traits::prelude::Tsify>::from_js(<Self as $crate::wasm_traits::prelude::Tsify>::JsType::from_abi(js));
                $crate::wasm_traits::prelude::UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err)
            }
        }
        impl $crate::wasm_traits::prelude::OptionFromWasmAbi for $type_name {
            #[inline]
            fn is_none(js: &Self::Abi) -> bool {
                <<Self as $crate::wasm_traits::prelude::Tsify>::JsType as $crate::wasm_traits::prelude::OptionFromWasmAbi>::is_none(js)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_complementary_wasm_traits {
    ($type_name:path) => {
        impl $crate::wasm_traits::prelude::RefFromWasmAbi for $type_name {
            type Abi = <$crate::wasm_traits::prelude::JsValue as $crate::wasm_traits::prelude::RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name as $crate::wasm_traits::prelude::FromWasmAbi>::from_abi(js))
            }
        }
        impl $crate::wasm_traits::prelude::LongRefFromWasmAbi for $type_name {
            type Abi = <$crate::wasm_traits::prelude::JsValue as $crate::wasm_traits::prelude::RefFromWasmAbi>::Abi;
            type Anchor = Box<$type_name>;
            unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new(<$type_name as $crate::wasm_traits::prelude::FromWasmAbi>::from_abi(js))
            }
        }
        impl $crate::wasm_traits::prelude::VectorIntoWasmAbi for $type_name {
            type Abi = <Box<[$crate::wasm_traits::prelude::JsValue]> as $crate::wasm_traits::prelude::IntoWasmAbi>::Abi;
            fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
                $crate::wasm_traits::prelude::js_value_vector_into_abi(vector)
            }
        }
        impl $crate::wasm_traits::prelude::VectorFromWasmAbi for $type_name {
            type Abi = <Box<[$crate::wasm_traits::prelude::JsValue]> as $crate::wasm_traits::prelude::IntoWasmAbi>::Abi;
            unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
                $crate::wasm_traits::prelude::js_value_vector_from_abi(js)
            }
        }
        impl $crate::wasm_traits::prelude::WasmDescribeVector for $type_name {
            fn describe_vector() {
                $crate::wasm_traits::prelude::inform($crate::wasm_traits::prelude::VECTOR);
                <$type_name as $crate::wasm_traits::prelude::WasmDescribe>::describe();
            }
        }
        impl From<$type_name> for $crate::wasm_traits::prelude::JsValue {
            fn from(value: $type_name) -> Self {
                let mut err = "".to_string();
                let result = $crate::wasm_traits::prelude::to_value(&value);
                $crate::wasm_traits::prelude::UnwrapThrowExt::expect_throw(result.inspect_err(|e| err.push_str(&e.to_string())), &err)
            }
        }
        impl $crate::wasm_traits::prelude::TryFromJsValue for $type_name {
            type Error = serde_wasm_bindgen::Error;
            fn try_from_js_value(value: $crate::wasm_traits::prelude::JsValue) -> Result<Self, Self::Error> {
                $crate::wasm_traits::prelude::from_value(value)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_all_wasm_traits {
    ($type_name:path) => {
        $crate::impl_main_wasm_traits!($type_name);
        $crate::impl_complementary_wasm_traits!($type_name);
    };
}
