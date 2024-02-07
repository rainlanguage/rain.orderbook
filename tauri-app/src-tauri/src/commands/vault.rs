use crate::error::CommandResult;
use crate::transaction_status::{SeriesPosition, TransactionStatusNoticeRwLock};
use rain_orderbook_common::{
    deposit::DepositArgs,
    subgraph::SubgraphArgs,
    transaction::TransactionArgs,
    withdraw::WithdrawArgs,
};
use rain_orderbook_subgraph_client::{
    types::{flattened::{TokenVaultFlattened, VaultBalanceChangeFlattened, TryIntoFlattenedError}, vault_balance_change::VaultBalanceChange, vault_detail, vaults_list},
    WriteCsv,
    PaginationArgs,
};
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub async fn vaults_list(
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<Vec<vaults_list::TokenVault>> {
    let vaults = subgraph_args
        .to_subgraph_client()
        .await?
        .vaults_list(pagination_args)
        .await?;
    Ok(vaults)
}

#[tauri::command]
pub async fn vaults_list_write_csv(
    path: PathBuf,
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<()> {
    let vaults = subgraph_args
        .to_subgraph_client()
        .await?
        .vaults_list(pagination_args)
        .await?;
    let vaults_flattened: Vec<TokenVaultFlattened> = vaults.into_iter().map(|o| o.into()).collect();
    vaults_flattened.write_csv(path)?;

    Ok(())
}

#[tauri::command]
pub async fn vault_list_balance_changes(
    id: String,
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<Vec<VaultBalanceChange>> {
    let data = subgraph_args
        .to_subgraph_client()
        .await?
        .vault_list_balance_changes(id.into(), pagination_args)
        .await?;
    Ok(data)
}

#[tauri::command]
pub async fn vault_list_balance_changes_write_csv(
    id: String,
    path: PathBuf,
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<()> {
    let data = subgraph_args
        .to_subgraph_client()
        .await?
        .vault_list_balance_changes(id.into(), pagination_args)
        .await?;
    let data_flattened: Vec<VaultBalanceChangeFlattened> = data.into_iter().map(|o| o.try_into()).collect::<Result<Vec<VaultBalanceChangeFlattened>, TryIntoFlattenedError>>()?;
    data_flattened.write_csv(path)?;

    Ok(())
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
