use alloy_ethers_typecast::transaction::WriteTransactionStatus;
use alloy_sol_types::SolCall;
use std::fmt::Debug;

pub fn display_write_transaction_status<T: SolCall + Debug>(status: WriteTransactionStatus<T>) {
    match status {
        WriteTransactionStatus::PendingPrepare(_) => {
            println!("⏳  Preparing transaction. Please wait.");
        }
        WriteTransactionStatus::PendingSign(_) => {
            println!("🖋   Please sign the transaction on your Ledger device.");
        }
        WriteTransactionStatus::PendingSend(_) => {
            println!("⏳  Awaiting transaction confirmation. Please wait.");
        }
        WriteTransactionStatus::Confirmed(receipt) => {
            println!("✅  Transaction confirmed: {:?}", receipt.transaction_hash);
        }
    }
}
