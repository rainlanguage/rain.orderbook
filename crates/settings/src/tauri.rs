use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{add_ts_content, prelude::*};

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub enum ToastMessageType {
    Success,
    Error,
    Warning,
    Info,
}

#[cfg(target_family = "wasm")]
add_ts_content!(
    r#"export interface ToastPayload {
	message_type: ToastMessageType;
	text: string;
}

export type TransactionStatus =
	| { type: "Initialized", payload?: undefined }
	| { type: "PendingPrepare", payload?: undefined }
	| { type: "PendingSign", payload?: undefined }
	| { type: "PendingSend", payload?: undefined }
	| { type: "Confirmed", payload: string }
	| { type: "Failed", payload: string };

export interface TransactionStatusNotice {
	id: string;
	status: TransactionStatus;
	created_at: string;
	/** Human-readable label to display in the UI, describing the transaction i.e. "Approving ERC20 Token Spend" */
	label: string;
}"#
);
