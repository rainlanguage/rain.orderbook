use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

#[typeshare]
#[derive(Serialize, Deserialize, Clone)]
pub enum ToastMessageType {
    Success,
    Error,
    Warning,
    Info,
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone)]
pub struct ToastPayload {
    pub message_type: ToastMessageType,
    pub text: String,
}

impl ToastPayload {
    pub fn emit(&self, app_handle: AppHandle) {
        let _ = app_handle.emit_all("toast", self);
    }
}

pub fn toast_error(app_handle: AppHandle, text: String) {
    let toast = ToastPayload {
        message_type: ToastMessageType::Error,
        text,
    };
    toast.emit(app_handle);
}
