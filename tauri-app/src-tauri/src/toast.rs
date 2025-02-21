use crate::types::{ToastMessageType, ToastPayload};
use tauri::{AppHandle, Manager};

impl ToastPayload {
    fn emit(&self, app_handle: AppHandle) {
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
