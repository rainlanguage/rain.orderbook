use alloy_ethers_typecast::transaction::WriteTransactionStatus;
use alloy_sol_types::SolCall;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, Manager};
use tsify::Tsify;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, Tsify)]
#[serde(tag = "type", content = "payload")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum TransactionStatus {
    Initialized,
    PendingPrepare,
    PendingSign,
    PendingSend,
    Confirmed(String),
    Failed(String),
}

impl<T: SolCall + Clone> From<WriteTransactionStatus<T>> for TransactionStatus {
    fn from(val: WriteTransactionStatus<T>) -> Self {
        match val {
            WriteTransactionStatus::PendingPrepare(_) => TransactionStatus::PendingPrepare,
            WriteTransactionStatus::PendingSign(_) => TransactionStatus::PendingSign,
            WriteTransactionStatus::PendingSend(_) => TransactionStatus::PendingSend,
            WriteTransactionStatus::Confirmed(receipt) => {
                TransactionStatus::Confirmed(format!("{:?}", receipt.transaction_hash))
            }
        }
    }
}

/// Position and Total number in a 'series' of transactions
/// i.e. position: 1, total: 2 would be represent "Transaction 1 of 2"
#[derive(Serialize, Deserialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SeriesPosition {
    pub position: u8,
    pub total: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TransactionStatusNotice {
    pub id: Uuid,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,

    /// Human-readable label to display in the UI, describing the transaction i.e. "Approving ERC20 Token Spend"
    pub label: String,
    pub series_position: Option<SeriesPosition>,
}

pub struct TransactionStatusNoticeRwLock(RwLock<TransactionStatusNotice>);

impl TransactionStatusNoticeRwLock {
    pub fn new(label: String, series_position: Option<SeriesPosition>) -> Self {
        let notice = TransactionStatusNotice {
            id: Uuid::new_v4(),
            status: TransactionStatus::Initialized,
            created_at: Utc::now(),
            label,
            series_position,
        };
        Self(RwLock::new(notice))
    }

    pub fn update_status_and_emit<T: SolCall + Clone>(
        &self,
        app_handle: AppHandle,
        status: WriteTransactionStatus<T>,
    ) {
        self.update_status(status);
        self.emit(app_handle);
    }

    pub fn set_failed_status_and_emit(&self, app_handle: AppHandle, message: String) {
        self.set_failed_status(message);
        self.emit(app_handle);
    }

    fn update_status<T: SolCall + Clone>(&self, status: WriteTransactionStatus<T>) {
        let mut notice = self.0.write().unwrap();
        notice.status = status.into();
    }

    fn set_failed_status(&self, message: String) {
        let mut notice = self.0.write().unwrap();
        notice.status = TransactionStatus::Failed(message);
    }

    fn emit(&self, app_handle: AppHandle) {
        app_handle
            .emit_all("transaction_status_notice", self.0.read().unwrap().clone())
            .unwrap();
    }
}
