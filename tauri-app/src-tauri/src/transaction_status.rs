use crate::types::{TransactionStatus, TransactionStatusNotice};
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::transaction::WriteTransactionStatus;
use chrono::Utc;
use std::sync::RwLock;
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

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

pub struct TransactionStatusNoticeRwLock(RwLock<TransactionStatusNotice>);

impl TransactionStatusNoticeRwLock {
    pub fn new(label: String) -> Self {
        let notice = TransactionStatusNotice {
            id: Uuid::new_v4(),
            status: TransactionStatus::Initialized,
            created_at: Utc::now(),
            label,
        };
        Self(RwLock::new(notice))
    }

    pub fn update_status_and_emit<T: SolCall + Clone, R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        status: WriteTransactionStatus<T>,
    ) {
        self.update_status(status);
        self.emit(app_handle);
    }

    pub fn set_failed_status_and_emit<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        message: String,
    ) {
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

    fn emit<R: Runtime>(&self, app_handle: &AppHandle<R>) {
        app_handle
            .emit_all("transaction_status_notice", self.0.read().unwrap().clone())
            .unwrap();
    }
}
