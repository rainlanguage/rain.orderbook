use serde::{Deserialize, Serialize};
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
