use alloy_ethers_typecast::ethers_address_to_alloy;
use rain_orderbook_bindings::{IOrderBookV3::depositCall, IERC20::approveCall};
use rain_orderbook_cli::transaction::ExecuteTransaction;
use rain_orderbook_common::{
    deposit::DepositArgs, subgraph::SubgraphArgs, transaction::TransactionArgs,
};
use rain_orderbook_subgraph_queries::types::{
    vault::TokenVault as VaultDetail, vaults::TokenVault as VaultsListItem,
};

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
    deposit_args: DepositArgs,
    transaction_args: TransactionArgs,
) -> Result<(), String> {
    // Prepare deposit call
    let deposit_call: depositCall = deposit_args
        .clone()
        .try_into()
        .map_err(|_| String::from("Deposit arguments invalid"))?;

    // Prepare approve call
    let mut execute_tx = ExecuteTransaction { transaction_args };
    let ledger_client = execute_tx.connect_ledger().await.map_err(|e| -> String {
        println!("connect_ledger error: {:?}", e);

        e.to_string()
    })?;
    let ledger_address = ethers_address_to_alloy(ledger_client.client.address());
    let approve_call: approveCall = deposit_args
        .clone()
        .try_into_approve_call(ledger_address)
        .map_err(|e| -> String {
            println!("try_into_approve_call error {:?}", e);

            e.to_string()
        })?;

    println!("Step 1/2: Approve token transfer");
    let receipt = execute_tx
        .send(ledger_client, approve_call)
        .await
        .map_err(|e| -> String {
            println!("send approve call error {:?}", e);

            e.to_string()
        })?;
    println!("recipt {:?}", receipt);

    println!("Step 2/2: Deposit tokens into vault");
    let ledger_client = execute_tx.connect_ledger().await.map_err(|e| -> String {
        println!("connect_ledger error {:?}", e);

        e.to_string()
    })?;

    let receipt = execute_tx
        .send(ledger_client, deposit_call)
        .await
        .map_err(|e| -> String {
            println!("send deposit call error {:?}", e);
            e.to_string()
        })?;

    println!("recipt {:?}", receipt);
    Ok(())
}
