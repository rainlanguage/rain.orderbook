use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "payload")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub enum TransactionStatus {
    Initialized,
    PendingPrepare,
    PendingSign,
    PendingSend,
    Confirmed(String),
    Failed(String),
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TransactionStatus);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TransactionStatusNotice {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub id: Uuid,
    pub status: TransactionStatus,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub created_at: DateTime<Utc>,
    /// Human-readable label to display in the UI, describing the transaction i.e. "Approving ERC20 Token Spend"
    pub label: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TransactionStatusNotice);
