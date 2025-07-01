use alloy::sol_types::SolCall;
use alloy_ethers_typecast::WriteTransactionStatus;
use std::fmt::Debug;
use tracing::info;

pub fn display_write_transaction_status<T: SolCall + Debug>(status: WriteTransactionStatus<T>) {
    match status {
        WriteTransactionStatus::PendingPrepare(_) => {
            info!("⏳  Preparing transaction. Please wait.");
        }
        WriteTransactionStatus::PendingSign(_) => {
            info!("🖋   Please sign the transaction on your Ledger device.");
        }
        WriteTransactionStatus::Sending => {
            info!("⏳  Awaiting transaction confirmation. Please wait.");
        }
        WriteTransactionStatus::Confirmed(receipt) => {
            info!("✅  Transaction confirmed: {:?}", receipt.transaction_hash);
        }
    }
}