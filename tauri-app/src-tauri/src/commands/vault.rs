use crate::toast::{ToastMessageType, ToastPayload};
use alloy_ethers_typecast::transaction::WriteTransactionStatus;
use alloy_sol_types::SolCall;
use rain_orderbook_common::{
    deposit::DepositArgs, subgraph::SubgraphArgs, transaction::TransactionArgs,
};
use rain_orderbook_subgraph_queries::types::{
    vault::TokenVault as VaultDetail, vaults::TokenVault as VaultsListItem,
};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn vaults_list(subgraph_args: SubgraphArgs) -> Result<Vec<VaultsListItem>, String> {
    subgraph_args
        .to_subgraph_client()
        .await
        .map_err(|_| String::from("Subgraph URL is invalid"))?
        .vaults()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vault_detail(id: String, subgraph_args: SubgraphArgs) -> Result<VaultDetail, String> {
    subgraph_args
        .to_subgraph_client()
        .await
        .map_err(|_| String::from("Subgraph URL is invalid"))?
        .vault(id.into())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vault_deposit(
    app_handle: AppHandle,
    deposit_args: DepositArgs,
    transaction_args: TransactionArgs,
) -> Result<(), String> {
    println!("----- Transaction (1/2): Approve ERC20 token spend -----");
    deposit_args
        .execute(
            transaction_args,
            |status| handle_write_transaction_status_changed(app_handle.clone(), status),
            |status| handle_write_transaction_status_changed(app_handle.clone(), status),
            || {
                println!("----- Transaction (2/2): Deposit tokens into Orderbook -----");
            },
        )
        .await
        .map_err(|e| toast_error(app_handle.clone(), format!("{}", e)))?;
    Ok(())
}

fn toast_error(app_handle: AppHandle, text: String) -> String {
    app_handle
        .emit_all(
            "toast",
            ToastPayload {
                text: text.clone(),
                message_type: ToastMessageType::Error,
            },
        )
        .unwrap();
    text
}

fn handle_write_transaction_status_changed<C: SolCall + Clone>(
    app_handle: AppHandle,
    status: WriteTransactionStatus<C>,
) {
    match status {
        WriteTransactionStatus::PendingPrepare(_) => {
            app_handle
                .emit_all(
                    "toast",
                    ToastPayload {
                        text: "Preparing Transaction".into(),
                        message_type: ToastMessageType::Info,
                    },
                )
                .unwrap();
        }
        WriteTransactionStatus::PendingSign(_) => {
            app_handle
                .emit_all(
                    "toast",
                    ToastPayload {
                        text: "Please review and sign the transaction on your Ledger device."
                            .into(),
                        message_type: ToastMessageType::Warning,
                    },
                )
                .unwrap();
        }
        WriteTransactionStatus::PendingSend(_) => {
            app_handle
                .emit_all(
                    "toast",
                    ToastPayload {
                        text: "Submitting transaction".into(),
                        message_type: ToastMessageType::Info,
                    },
                )
                .unwrap();
        }
        WriteTransactionStatus::Confirmed(receipt) => {
            app_handle
                .emit_all(
                    "toast",
                    ToastPayload {
                        text: format!("Transaction confirmed: {}", receipt.transaction_hash),
                        message_type: ToastMessageType::Success,
                    },
                )
                .unwrap();
        }
    }
}
