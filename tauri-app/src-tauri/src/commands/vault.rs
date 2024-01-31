use crate::error::CommandResult;
use crate::transaction_status::{SeriesPosition, TransactionStatusNoticeRwLock};
use rain_orderbook_common::{
    deposit::DepositArgs,
    subgraph::{SubgraphArgs, SubgraphPaginationArgs},
    transaction::TransactionArgs,
    withdraw::WithdrawArgs,
};
use rain_orderbook_subgraph_client::types::{vault_detail, vaults_list};
use tauri::AppHandle;

#[tauri::command]
pub async fn vaults_list(
    subgraph_args: SubgraphArgs,
    pagination_args: SubgraphPaginationArgs,
) -> CommandResult<Vec<vaults_list::TokenVault>> {
    let vaults = subgraph_args
        .to_subgraph_client()
        .await?
        .vaults_list(pagination_args)
        .await?;
    Ok(vaults)
}

#[tauri::command]
pub async fn vault_detail(
    id: String,
    subgraph_args: SubgraphArgs,
) -> CommandResult<vault_detail::TokenVault> {
    let vault = subgraph_args
        .to_subgraph_client()
        .await?
        .vault_detail(id.into())
        .await?;

    Ok(vault)
}

#[tauri::command]
pub async fn vault_deposit(
    app_handle: AppHandle,
    deposit_args: DepositArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
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
) -> CommandResult<()> {
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
