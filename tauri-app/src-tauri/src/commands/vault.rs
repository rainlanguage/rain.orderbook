use crate::transaction_status::{SeriesPosition, TransactionStatusNoticeRwLock};
use rain_orderbook_common::{
    deposit::DepositArgs, subgraph::SubgraphArgs, transaction::TransactionArgs,
    withdraw::WithdrawArgs,
};
use rain_orderbook_subgraph_queries::types::{
    vault::TokenVault as VaultDetail, vaults::TokenVault as VaultsListItem,
};
use tauri::AppHandle;

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
    let tx_status_notice = TransactionStatusNoticeRwLock::new(
        "Approve ERC20 token transfer".into(),
        Some(SeriesPosition {
            position: 1,
            total: 2,
        }),
    );
    let _ = deposit_args
        .execute_approve(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(app_handle.clone(), status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(app_handle.clone(), e.to_string());
        });

    let tx_status_notice = TransactionStatusNoticeRwLock::new(
        "Deposit tokens into vault".into(),
        Some(SeriesPosition {
            position: 2,
            total: 2,
        }),
    );
    let _ = deposit_args
        .execute_deposit(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(app_handle.clone(), status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(app_handle.clone(), e.to_string());
        });

    Ok(())
}

#[tauri::command]
pub async fn vault_withdraw(
    app_handle: AppHandle,
    withdraw_args: WithdrawArgs,
    transaction_args: TransactionArgs,
) -> Result<(), String> {
    let tx_status_notice =
        TransactionStatusNoticeRwLock::new("Withdraw tokens from vault".into(), None);
    let _ = withdraw_args
        .execute(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(app_handle.clone(), status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(app_handle.clone(), e.to_string());
        });

    Ok(())
}
