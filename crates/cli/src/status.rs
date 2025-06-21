use alloy::sol_types::SolCall;
use alloy_ethers_typecast::transaction::WriteTransactionStatus;
use std::fmt::Debug;
use tracing::info;

pub fn display_write_transaction_status<T: SolCall + Debug>(status: WriteTransactionStatus<T>) {
    match status {
        WriteTransactionStatus::PendingPrepare(_) => {
            info!("⏳  Preparing transaction. Please wait.");
        }
        WriteTransactionStatus::PendingSignAndSend(_) => {
            info!("🖋   Please sign the transaction on your Ledger device. Once signed, the transaction will be sent.");
        }
        WriteTransactionStatus::Confirmed(receipt) => {
            info!("✅  Transaction confirmed: {:?}", receipt.transaction_hash);
        }
    }
}
