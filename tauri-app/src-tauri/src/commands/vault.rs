use alloy::primitives::Bytes;
use alloy::sol_types::SolCall;
use rain_math_float::FloatError;
use rain_orderbook_bindings::{IOrderBookV5::deposit3Call, IERC20::approveCall};
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
use tauri::{AppHandle, Runtime};

use crate::error::CommandResult;
use crate::toast::toast_error;
use crate::transaction_status::TransactionStatusNoticeRwLock;

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
    chain_id: u32,
    deposit_args: DepositArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
    let tx_status_notice =
        TransactionStatusNoticeRwLock::new("Approve ERC20 token transfer".into(), chain_id);
    let _ = deposit_args
        .execute_approve(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(&app_handle, status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(&app_handle, e.to_string());
        });

    let tx_status_notice =
        TransactionStatusNoticeRwLock::new("Deposit tokens into vault".into(), chain_id);
    let _ = deposit_args
        .execute_deposit(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(&app_handle, status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(&app_handle, e.to_string());
        });

    Ok(())
}

#[tauri::command]
pub async fn vault_deposit_approve_calldata(
    deposit_args: DepositArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<Bytes> {
    let calldata = approveCall {
        spender: transaction_args.orderbook_address,
        amount: deposit_args.amount,
    }
    .abi_encode();

    Ok(Bytes::from(calldata))
}

#[tauri::command]
pub async fn vault_deposit_calldata<R: Runtime>(
    app_handle: AppHandle<R>,
    deposit_args: DepositArgs,
) -> CommandResult<Bytes> {
    let deposit_call: deposit3Call = deposit_args.try_into().inspect_err(|e: &FloatError| {
        toast_error(&app_handle, e.to_string());
    })?;
    Ok(Bytes::from(deposit_call.abi_encode()))
}

#[tauri::command]
pub async fn vault_withdraw(
    app_handle: AppHandle,
    chain_id: u32,
    withdraw_args: WithdrawArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
    let tx_status_notice =
        TransactionStatusNoticeRwLock::new("Withdraw tokens from vault".into(), chain_id);
    let _ = withdraw_args
        .execute(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(&app_handle, status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(&app_handle, e.to_string());
        });

    Ok(())
}

#[tauri::command]
pub async fn vault_withdraw_calldata<R: Runtime>(
    app_handle: AppHandle<R>,
    withdraw_args: WithdrawArgs,
) -> CommandResult<Bytes> {
    let calldata = withdraw_args
        .get_withdraw_calldata()
        .await
        .inspect_err(|e| {
            toast_error(&app_handle, e.to_string());
        })?;

    Ok(Bytes::from(calldata))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CommandError;
    use alloy::{
        primitives::{Address, B256, U256},
        sol_types::SolCall,
    };
    use httpmock::MockServer;
    use rain_math_float::Float;
    use rain_orderbook_bindings::{
        IOrderBookV5::{deposit3Call, withdraw3Call},
        IERC20::approveCall,
    };
    use rain_orderbook_subgraph_client::utils::float::*;
    use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
    use serde_json::{json, Value};
    use tauri::Manager;

    fn get_vault1_json() -> Value {
        json!({
          "id": "vault1",
          "owner": "0x0000000000000000000000000000000000000000",
          "vaultId": "0x10",
          "balance": F1,
          "token": {
            "id": "token1",
            "address": "0x0000000000000000000000000000000000000000",
            "name": "Token 1",
            "symbol": "TKN1",
            "decimals": "18"
          },
          "orderbook": {
            "id": "0x0000000000000000000000000000000000000000"
          },
          "ordersAsOutput": [],
          "ordersAsInput": [],
          "balanceChanges": []
        })
    }
    fn get_vault2_json() -> Value {
        json!({
            "id": "vault2",
            "owner": "0x0000000000000000000000000000000000000000",
            "vaultId": "0x20",
            "balance": F2,
            "token": {
                "id": "token2",
                "address": "0x0000000000000000000000000000000000000000",
                "name": "Token 2",
                "symbol": "TKN2",
                "decimals": "18"
            },
            "orderbook": {
                "id": "0x0000000000000000000000000000000000000000"
            },
            "ordersAsOutput": [],
            "ordersAsInput": [],
            "balanceChanges": []
        })
    }

    #[tokio::test]
    async fn test_vaults_list_write_csv() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "vaults": [
                        get_vault1_json(),
                        get_vault2_json(),
                    ]
                }
            }));
        });
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":200");
            then.status(200).json_body_obj(&json!({
                "data": { "vaults": [] }
            }));
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("./test.csv");

        vaults_list_write_csv(
            path.clone(),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
        )
        .await
        .unwrap();

        let expected_content = "
id,owner,vault_id,token_name,token_symbol,token_decimals,token_address,balance_display,balance
vault1,0x0000000000000000000000000000000000000000,0x10,Token 1,TKN1,18,0x0000000000000000000000000000000000000000,1,0x0000000000000000000000000000000000000000000000000000000000000001
vault2,0x0000000000000000000000000000000000000000,0x20,Token 2,TKN2,18,0x0000000000000000000000000000000000000000,2,0x0000000000000000000000000000000000000000000000000000000000000002
";

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content.trim(), expected_content.trim());
    }

    #[tokio::test]
    async fn test_vaults_list_write_csv_empty_response() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "vaults": []
                }
            }));
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("./test.csv");

        let res = vaults_list_write_csv(
            path.clone(),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
        )
        .await;
        assert!(res.is_ok());

        let expected_content = "";
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, expected_content);
    }

    #[tokio::test]
    async fn test_vaults_list_write_csv_invalid_rpc_url() {
        let err = vaults_list_write_csv(
            PathBuf::from("./test.csv"),
            SubgraphArgs {
                url: "invalid_url".to_string(),
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            CommandError::URLParseError(url::ParseError::RelativeUrlWithoutBase)
        ));
    }

    #[tokio::test]
    async fn test_vaults_list_write_csv_malformed_response() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({}));
        });

        let err = vaults_list_write_csv(
            PathBuf::from("./test.csv"),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            CommandError::OrderbookSubgraphClientError(
                OrderbookSubgraphClientError::CynicClientError(_)
            )
        ));
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_write_csv() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "vaultBalanceChanges": [
                        {
                            "__typename": "Deposit",
                            "amount": F5,
                            "newVaultBalance": F5,
                            "oldVaultBalance": F0,
                            "vault": {
                                "id": "0x166aeed725f0f3ef9fe62f2a9054035756d55e5560b17afa1ae439e9cd362902",
                                "vaultId": "1",
                                "token": {
                                    "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                    "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                    "name": "Wrapped Flare",
                                    "symbol": "WFLR",
                                    "decimals": "18"
                                }
                            },
                            "timestamp": "1734054063",
                            "transaction": {
                                "id": "0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22",
                                "from": "0x7177b9d00bb5dbcaaf069cc63190902763783b09",
                                "blockNumber": "34407047",
                                "timestamp": "1734054063"
                            },
                            "orderbook": {
                                "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                            }
                        }
                    ]
                }
            }));
        });

        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":200");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "vaultBalanceChanges": []
                }
            }));
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("./test.csv");

        vault_balance_changes_list_write_csv(
            "id".to_string(),
            path.clone(),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
        )
        .await
        .unwrap();

        let expected_content = "
timestamp,timestamp_display,from,amount,amount_display_signed,change_type_display,balance
1734054063,2024-12-13 01:41:03 UTC,0x7177b9d00bb5dbcaaf069cc63190902763783b09,0x0000000000000000000000000000000000000000000000000000000000000005,5,Deposit,0x0000000000000000000000000000000000000000000000000000000000000005
";
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content.trim(), expected_content.trim());
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_write_csv_empty_response() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "vaultBalanceChanges": []
                }
            }));
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("./test.csv");

        let res = vault_balance_changes_list_write_csv(
            "id".to_string(),
            path.clone(),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
        )
        .await;
        assert!(res.is_ok());

        let expected_content = "";
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, expected_content);
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_write_csv_invalid_rpc_url() {
        let err = vault_balance_changes_list_write_csv(
            "id".to_string(),
            PathBuf::from("./test.csv"),
            SubgraphArgs {
                url: "invalid_url".to_string(),
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            CommandError::URLParseError(url::ParseError::RelativeUrlWithoutBase)
        ));
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_write_csv_malformed_response() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({}));
        });

        let err = vault_balance_changes_list_write_csv(
            "id".to_string(),
            PathBuf::from("./test.csv"),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            CommandError::OrderbookSubgraphClientError(
                OrderbookSubgraphClientError::PaginationClientError(_)
            )
        ));
    }

    #[tokio::test]
    async fn test_vault_deposit_approve_calldata() {
        let amount = U256::from(50);
        let decimals = 18;

        let res = vault_deposit_approve_calldata(
            DepositArgs {
                token: Address::default(),
                vault_id: B256::from(U256::from(1)),
                amount,
                decimals,
            },
            TransactionArgs {
                orderbook_address: Address::default(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let expected: Bytes = approveCall {
            spender: Address::default(),
            amount,
        }
        .abi_encode()
        .into();

        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_vault_deposit_calldata() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let amount = U256::from(50);
        let decimals = 18;

        let res = vault_deposit_calldata(
            app_handle.clone(),
            DepositArgs {
                token: Address::default(),
                vault_id: B256::from(U256::from(1)),
                amount,
                decimals,
            },
        )
        .await
        .unwrap();

        let Float(amount) = Float::from_fixed_decimal(amount, decimals).unwrap();

        let expected: Bytes = deposit3Call {
            token: Address::default(),
            depositAmount: amount,
            vaultId: B256::from(U256::from(1)),
            tasks: vec![],
        }
        .abi_encode()
        .into();

        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_vault_withdraw_calldata() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let amount = U256::from(50);
        let decimals = 18;
        let target_amount = Float::from_fixed_decimal(amount, decimals).unwrap();

        let res = vault_withdraw_calldata(
            app_handle.clone(),
            WithdrawArgs {
                token: Address::default(),
                vault_id: B256::from(U256::from(1)),
                target_amount,
            },
        )
        .await
        .unwrap();

        let Float(target_amount_bytes) = target_amount;

        let expected: Bytes = withdraw3Call {
            token: Address::default(),
            targetAmount: target_amount_bytes,
            vaultId: B256::from(U256::from(1)),
            tasks: vec![],
        }
        .abi_encode()
        .into();

        assert_eq!(res, expected);
    }
}
