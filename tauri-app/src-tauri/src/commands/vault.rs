use crate::toast::{ToastMessageType, ToastPayload};
use alloy_ethers_typecast::ethers_address_to_alloy;
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
    println!("Step 1/2: Approve token transfer");
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
    let approve_call: approveCall = deposit_args.clone().into_approve_call(ledger_address);
    let send_future = execute_tx.send(ledger_client, approve_call);

    app_handle
        .emit_all(
            "toast",
            ToastPayload {
                text: "Approve the token transfer call on your Ledger device".into(),
                message_type: ToastMessageType::Info,
            },
        )
        .unwrap();

    let receipt = send_future.await.map_err(|e| -> String {
        println!("error : {:?}", e);
        let text = e.to_string();
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

    app_handle
        .emit_all(
            "toast",
            ToastPayload {
                text: format!("Transaction Success: {}", receipt.transaction_hash),
                message_type: ToastMessageType::Success,
            },
        )
        .unwrap();
    println!("recipt {:?}", receipt);

    // Call OrderbookV3 deposit
    println!("Step 2/2: Deposit tokens into vault");
    let deposit_call: depositCall = deposit_args
        .clone()
        .try_into()
        .map_err(|_| String::from("Deposit arguments invalid"))?;
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

    let send_future = execute_tx.send(ledger_client, deposit_call);

    app_handle
        .emit_all(
            "toast",
            ToastPayload {
                text: "Approve the deposit call on your Ledger device".into(),
                message_type: ToastMessageType::Info,
            },
        )
        .unwrap();

    let receipt = send_future.await.map_err(|e| -> String {
        println!("error : {:?}", e);
        let text = e.to_string();
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

    app_handle
        .emit_all(
            "toast",
            ToastPayload {
                text: format!("Transaction Hash: {}", receipt.transaction_hash),
                message_type: ToastMessageType::Success,
            },
        )
        .unwrap();

    println!("recipt {:?}", receipt);
    Ok(())
}
