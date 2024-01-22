use crate::toast::{ToastMessageType, ToastPayload};
use alloy_ethers_typecast::ethers_address_to_alloy;
use alloy_ethers_typecast::transaction::{WriteTransaction, WriteTransactionStatus};
use alloy_sol_types::SolCall;
use rain_orderbook_bindings::{IOrderBookV3::depositCall, IERC20::approveCall};
use rain_orderbook_cli::transaction::ExecuteTransaction;
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
    let mut execute_tx = ExecuteTransaction {
        transaction_args: transaction_args.clone(),
    };

    // Connect to Ledger device
    let ledger_client = execute_tx.connect_ledger().await.map_err(|e| -> String {
        println!("error : {:?}", e);

        let text = format!(
            "Unlock your Ledger device and open the app for chain {}",
            transaction_args.clone().chain_id
        );
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
    })?;

    // Call ERC20 approve
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
    let approve_call: approveCall = deposit_args.clone().into_approve_call(ledger_address);
    let call_params = transaction_args
        .to_write_contract_parameters(approve_call)
        .map_err(|e| e.to_string())?;
    WriteTransaction::new(ledger_client.client, call_params, 4, |status| {
        handle_write_transaction_status_changed(app_handle.clone(), status)
    })
    .execute()
    .await
    .map_err(|e| toast_error(app_handle.clone(), format!("Error: {}", e.to_string())))?;

    // Call OrderbookV3 deposit
    let deposit_call: depositCall = deposit_args
        .clone()
        .try_into()
        .map_err(|_| toast_error(app_handle.clone(), "Failed to construct depositCall".into()))?;
    let ledger_client = execute_tx.connect_ledger().await.map_err(|e| -> String {
        println!("error : {:?}", e);

        let text = format!("Ledger error: {:?}", e);
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
    })?;
    let call_params = transaction_args
        .to_write_contract_parameters(deposit_call)
        .map_err(|e| e.to_string())?;
    WriteTransaction::new(ledger_client.client, call_params, 4, |status| {
        handle_write_transaction_status_changed(app_handle.clone(), status)
    })
    .execute()
    .await
    .map_err(|e| toast_error(app_handle.clone(), format!("Error: {}", e.to_string())))?;

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
                        text: "Sending transaction".into(),
                        message_type: ToastMessageType::Info,
                    },
                )
                .unwrap();
        }
        WriteTransactionStatus::PendingConfirm => {
            app_handle
                .emit_all(
                    "toast",
                    ToastPayload {
                        text: "Awaiting transaction confirmations".into(),
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
