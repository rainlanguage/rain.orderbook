use crate::error::CommandResult;
use crate::toast::toast_error;
use crate::transaction_status::TransactionStatusNoticeRwLock;
use alloy::primitives::Bytes;
use rain_orderbook_common::{
    csv::TryIntoCsv,
    deposit::DepositArgs,
    subgraph::SubgraphArgs,
    transaction::TransactionArgs,
    types::{FlattenError, TokenVaultFlattened, VaultBalanceChangeFlattened},
    withdraw::WithdrawArgs,
};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub async fn vaults_list_write_csv(
    path: PathBuf,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let vaults = subgraph_args
        .to_subgraph_client()?
        .vaults_list_all()
        .await?;
    let vaults_flattened: Vec<TokenVaultFlattened> =
        vaults
            .into_iter()
            .map(|o| o.try_into())
            .collect::<Result<Vec<TokenVaultFlattened>, FlattenError>>()?;
    let csv_text = vaults_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}

#[tauri::command]
pub async fn vault_balance_changes_list_write_csv(
    id: String,
    path: PathBuf,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let data = subgraph_args
        .to_subgraph_client()?
        .vault_balance_changes_list_all(id.into())
        .await?;
    let data_flattened: Vec<VaultBalanceChangeFlattened> =
        data.into_iter()
            .map(|o| o.try_into())
            .collect::<Result<Vec<VaultBalanceChangeFlattened>, FlattenError>>()?;
    let csv_text = data_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}

#[tauri::command]
pub async fn vault_deposit(
    app_handle: AppHandle,
    deposit_args: DepositArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
    let tx_status_notice =
        TransactionStatusNoticeRwLock::new("Approve ERC20 token transfer".into());
    let _ = deposit_args
        .execute_approve(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(app_handle.clone(), status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(app_handle.clone(), e.to_string());
        });

    let tx_status_notice = TransactionStatusNoticeRwLock::new("Deposit tokens into vault".into());
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
pub async fn vault_deposit_approve_calldata(
    app_handle: AppHandle,
    deposit_args: DepositArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<Bytes> {
    let calldata = deposit_args
        .get_approve_calldata(transaction_args)
        .await
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;
    Ok(Bytes::from(calldata))
}

#[tauri::command]
pub async fn vault_deposit_calldata(
    app_handle: AppHandle,
    deposit_args: DepositArgs,
) -> CommandResult<Bytes> {
    let calldata = deposit_args.get_deposit_calldata().await.map_err(|e| {
        toast_error(app_handle.clone(), e.to_string());
        e
    })?;

    Ok(Bytes::from(calldata))
}

#[tauri::command]
pub async fn vault_withdraw(
    app_handle: AppHandle,
    withdraw_args: WithdrawArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
    let tx_status_notice = TransactionStatusNoticeRwLock::new("Withdraw tokens from vault".into());
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

#[tauri::command]
pub async fn vault_withdraw_calldata(
    app_handle: AppHandle,
    withdraw_args: WithdrawArgs,
) -> CommandResult<Bytes> {
    let calldata = withdraw_args.get_withdraw_calldata().await.map_err(|e| {
        toast_error(app_handle.clone(), e.to_string());
        e
    })?;

    Ok(Bytes::from(calldata))
}
