use crate::types::{TransactionStatus, TransactionStatusNotice};
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::WriteTransactionStatus;
use chrono::Utc;
use std::sync::RwLock;
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

impl<T: SolCall + Clone> From<WriteTransactionStatus<T>> for TransactionStatus {
    fn from(val: WriteTransactionStatus<T>) -> Self {
        match val {
            WriteTransactionStatus::PendingPrepare(_) => TransactionStatus::PendingPrepare,
            WriteTransactionStatus::PendingSign(_) => TransactionStatus::PendingSign,
            WriteTransactionStatus::Sending => TransactionStatus::Sending,
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, B256};
    use alloy_ethers_typecast::WriteContractParameters;
    use rain_math_float::Float;
    use rain_orderbook_bindings::IOrderBookV5::deposit3Call;

    #[test]
    fn test_new() {
        let notice = TransactionStatusNoticeRwLock::new("test".to_string());
        assert_eq!(
            notice.0.read().unwrap().status,
            TransactionStatus::Initialized
        );
        assert_eq!(notice.0.read().unwrap().label, "test");
    }

    #[test]
    fn test_update_status() {
        let app = tauri::test::mock_app();
        let notice = TransactionStatusNoticeRwLock::new("test".to_string());

        let Float(zero) = Float::parse("0".to_string()).unwrap();

        notice.update_status_and_emit(
            &app.handle(),
            WriteTransactionStatus::PendingPrepare(Box::new(WriteContractParameters {
                call: deposit3Call {
                    token: Address::ZERO,
                    vaultId: B256::ZERO,
                    depositAmount: zero,
                    tasks: Vec::new(),
                },
                address: Address::ZERO,
                gas: None,
                gas_price: None,
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                nonce: None,
                value: None,
            })),
        );
        assert_eq!(
            notice.0.read().unwrap().status,
            TransactionStatus::PendingPrepare
        );
    }

    #[test]
    fn test_set_failed_status() {
        let app = tauri::test::mock_app();
        let notice = TransactionStatusNoticeRwLock::new("test".to_string());

        notice.set_failed_status_and_emit(&app.handle(), "failed".to_string());
        assert_eq!(
            notice.0.read().unwrap().status,
            TransactionStatus::Failed("failed".to_string())
        );
    }
}
