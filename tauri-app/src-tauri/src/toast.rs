use crate::types::{ToastMessageType, ToastPayload};
use tauri::{AppHandle, Manager, Runtime};

impl ToastPayload {
    fn emit<R: Runtime>(&self, app_handle: &AppHandle<R>) {
        let _ = app_handle.emit_all("toast", self);
    }
}

pub fn toast_error<R: Runtime>(app_handle: &AppHandle<R>, text: String) {
    let toast = ToastPayload {
        message_type: ToastMessageType::Error,
        text,
    };
    toast.emit(app_handle);
}
