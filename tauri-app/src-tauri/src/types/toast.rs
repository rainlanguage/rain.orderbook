use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub enum ToastMessageType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ToastPayload {
    pub message_type: ToastMessageType,
    pub text: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ToastPayload);
