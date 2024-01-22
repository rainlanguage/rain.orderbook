use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum ToastMessageType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ToastPayload {
    pub message_type: ToastMessageType,
    pub text: String,
}
