use super::SubgraphError;
use alloy::primitives::{Address, Bytes, U256};
use cynic::Id;
use rain_orderbook_common::deposit::DepositArgs;
use rain_orderbook_common::transaction::TransactionArgs;
use rain_orderbook_common::withdraw::WithdrawArgs;
use rain_orderbook_subgraph_client::types::common::{
    SgOrder, SgVault, SgVaultBalanceChangeUnwrapped, SgVaultWithSubgraphName,
    SgVaultsListFilterArgs,
};
use rain_orderbook_subgraph_client::{
    MultiOrderbookSubgraphClient, MultiSubgraphArgs, OrderbookSubgraphClient, SgPaginationArgs,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct VaultCalldataResult(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(VaultCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetVaultsResult(
    #[tsify(type = "SgVaultWithSubgraphName[]")] Vec<SgVaultWithSubgraphName>,
);
impl_wasm_traits!(GetVaultsResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetVaultBalanceChangesResult(
    #[tsify(type = "SgVaultBalanceChangeUnwrapped[]")] Vec<SgVaultBalanceChangeUnwrapped>,
);
impl_wasm_traits!(GetVaultBalanceChangesResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct VaultAllowanceResult(#[tsify(type = "string")] U256);
impl_wasm_traits!(VaultAllowanceResult);

/// Fetches vault data from multiple subgraphs across different networks.
///
/// Queries multiple subgraphs simultaneously to retrieve vault information
/// across different blockchain networks.
///
/// # Parameters
///
/// * `subgraphs` - Array of subgraph configurations, each containing:
///   - `url`: Subgraph endpoint URL
///   - `name`: Human-readable network name for identification
/// * `filter_args` - Filtering options including:
///   - `owners`: Array of owner addresses to filter by (empty for all)
///   - `hide_zero_balance`: Whether to exclude vaults with zero balance
/// * `pagination_args` - Pagination configuration:
///   - `page`: Page number (1-based)
///   - `page_size`: Number of vaults per page
///
/// # Returns
///
/// * `Ok(GetVaultsResult)` - Array of vaults with their associated subgraph network names
/// * `Err(SubgraphError)` - Network errors, invalid parameters, or query failures
///
/// # Examples
///
/// ```javascript
/// const result = await getVaults(
///   [
///     { url: "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-polygon", name: "polygon" },
///     { url: "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-flare", name: "flare" }
///   ],
///   {
///     owners: ["0x1234567890abcdef1234567890abcdef12345678"],
///     hide_zero_balance: true
///   },
///   { page: 1, page_size: 25 }
/// );
/// if (result.error) {
///   console.error("Error fetching vaults:", result.error.readableMsg);
///   return;
/// }
/// const vaults = result.value;
/// // Do something with the vaults
/// ```
#[wasm_export(js_name = "getVaults", unchecked_return_type = "GetVaultsResult")]
pub async fn get_vaults(
    subgraphs: Vec<MultiSubgraphArgs>,
    filter_args: SgVaultsListFilterArgs,
    pagination_args: SgPaginationArgs,
) -> Result<GetVaultsResult, SubgraphError> {
    let client = MultiOrderbookSubgraphClient::new(subgraphs);
    Ok(GetVaultsResult(
        client.vaults_list(filter_args, pagination_args).await,
    ))
}

/// Fetches detailed information for a specific vault.
///
/// Retrieves complete vault information including token details, balance,
/// and associated orders.
///
/// # Parameters
///
/// * `url` - Subgraph endpoint URL
/// * `id` - Unique vault identifier
///
/// # Returns
///
/// * `Ok(SgVault)` - Complete vault information
/// * `Err(SubgraphError)` - Network or query errors
///
/// # Examples
///
/// ```javascript
/// const result = await getVault(
///   "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-polygon",
///   "0x1234567890abcdef1234567890abcdef12345678"
/// );
/// if (result.error) {
///   console.error("Vault not found:", result.error.readableMsg);
///   return;
/// }
/// const vault = result.value;
/// // Do something with the vault
/// ```
#[wasm_export(js_name = "getVault", unchecked_return_type = "SgVault")]
pub async fn get_vault(url: &str, id: &str) -> Result<SgVault, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(client.vault_detail(Id::new(id)).await?)
}

/// Fetches balance change history for a vault.
///
/// Retrieves chronological list of deposits, withdrawals, and trades affecting
/// a vault's balance.
///
/// # Parameters
///
/// * `url` - Subgraph endpoint URL
/// * `id` - Vault identifier
/// * `pagination_args` - Pagination configuration:
///   - `page`: Page number (1-based)
///   - `page_size`: Number of balance changes per page
///
/// # Returns
///
/// * `Ok(GetVaultBalanceChangesResult)` - Array of balance change events
/// * `Err(SubgraphError)` - Network or query errors
///
/// # Examples
///
/// ```javascript
/// const result = await getVaultBalanceChanges(
///   "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-polygon",
///   "vault_id_123",
///   { page: 1, page_size: 20 }
/// );
/// if (result.error) {
///   console.error("Error fetching history:", result.error.readableMsg);
///   return;
/// }
/// const changes = result.value;
/// // Do something with the changes
/// ```
#[wasm_export(
    js_name = "getVaultBalanceChanges",
    unchecked_return_type = "GetVaultBalanceChangesResult"
)]
pub async fn get_vault_balance_changes(
    url: &str,
    id: &str,
    pagination_args: SgPaginationArgs,
) -> Result<GetVaultBalanceChangesResult, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(GetVaultBalanceChangesResult(
        client
            .vault_balance_changes_list(Id::new(id), pagination_args)
            .await?,
    ))
}

/// Generates transaction calldata for depositing tokens into a vault.
///
/// Creates the contract calldata needed to deposit a specified amount of tokens
/// into a vault.
///
/// # Parameters
///
/// * `vault` - Target vault object containing:
///   - `token.address`: ERC20 token contract address
///   - `vault_id`: Unique vault identifier
///   - `orderbook.id`: Orderbook contract address for transaction target
/// * `deposit_amount` - Amount to deposit in token's smallest unit (e.g., "1000000000000000000" for 1 token with 18 decimals)
///
/// # Returns
///
/// * `Ok(VaultCalldataResult)` - Encoded transaction calldata as hex string
/// * `Err(SubgraphError)` - When deposit amount is zero/invalid or vault configuration is malformed
///
/// # Examples
///
/// ```javascript
/// const result = await getVaultDepositCalldata(
///   vault,
///   "1000000000000000000"
/// );
/// if (result.error) {
///   console.error("Cannot generate deposit:", result.error.readableMsg);
///   return;
/// }
/// const calldata = result.value;
/// // Do something with the calldata
/// ```
#[wasm_export(
    js_name = "getVaultDepositCalldata",
    unchecked_return_type = "VaultCalldataResult"
)]
pub async fn get_vault_deposit_calldata(
    vault: &SgVault,
    deposit_amount: &str,
) -> Result<VaultCalldataResult, SubgraphError> {
    let deposit_amount = validate_amount(deposit_amount)?;

    let deposit_args = DepositArgs {
        token: Address::from_str(&vault.token.address.0)?,
        vault_id: U256::from_str(&vault.vault_id.0)?,
        amount: deposit_amount,
    };

    Ok(VaultCalldataResult(Bytes::copy_from_slice(
        &deposit_args.get_deposit_calldata().await?,
    )))
}

/// Generates transaction calldata for withdrawing tokens from a vault.
///
/// Creates the contract calldata needed to withdraw a specified amount of tokens
/// from a vault.
///
/// # Parameters
///
/// * `vault` - Source vault object
/// * `withdraw_amount` - Amount to withdraw in token's smallest unit
///
/// # Returns
///
/// * `Ok(VaultCalldataResult)` - Encoded transaction calldata as hex string
/// * `Err(SubgraphError)` - Invalid amount or encoding errors
///
/// # Examples
///
/// ```javascript
/// const result = await getVaultWithdrawCalldata(
///   vault,
///   "500000000000000000"
/// );
/// if (result.error) {
///   console.error("Cannot generate withdrawal:", result.error.readableMsg);
///   return;
/// }
/// const calldata = result.value;
/// // Do something with the calldata
/// ```
#[wasm_export(
    js_name = "getVaultWithdrawCalldata",
    unchecked_return_type = "VaultCalldataResult"
)]
pub async fn get_vault_withdraw_calldata(
    vault: &SgVault,
    withdraw_amount: &str,
) -> Result<VaultCalldataResult, SubgraphError> {
    let withdraw_amount = validate_amount(withdraw_amount)?;

    Ok(VaultCalldataResult(Bytes::copy_from_slice(
        &WithdrawArgs {
            token: Address::from_str(&vault.token.address.0)?,
            vault_id: U256::from_str(&vault.vault_id.0)?,
            target_amount: withdraw_amount,
        }
        .get_withdraw_calldata()
        .await?,
    )))
}

/// Generates ERC20 approval calldata for vault deposits.
///
/// Creates the contract calldata needed to approve the orderbook contract to spend
/// tokens for a vault deposit, but only if additional approval is needed.
///
/// # Parameters
///
/// * `rpc_url` - Blockchain RPC endpoint for checking current allowance
/// * `vault` - Target vault object
/// * `deposit_amount` - Amount requiring approval
///
/// # Returns
///
/// * `Ok(VaultCalldataResult)` - Encoded approval calldata
/// * `Err(SubgraphError)` - Sufficient allowance exists or other errors
///
/// # Examples
///
/// ```javascript
/// const result = await getVaultApprovalCalldata(
///   "https://polygon-rpc.com",
///   vault,
///   "2000000000000000000"
/// );
/// if (result.error) {
///   console.error("Approval error:", result.error.readableMsg);
///   return;
/// }
/// const calldata = result.value;
/// // Do something with the calldata
/// ```
#[wasm_export(
    js_name = "getVaultApprovalCalldata",
    unchecked_return_type = "VaultCalldataResult"
)]
pub async fn get_vault_approval_calldata(
    rpc_url: &str,
    vault: &SgVault,
    deposit_amount: &str,
) -> Result<VaultCalldataResult, SubgraphError> {
    let deposit_amount = validate_amount(deposit_amount)?;
    let owner = Address::from_str(&vault.owner.0)?;

    let (deposit_args, transaction_args) =
        get_deposit_and_transaction_args(rpc_url, vault, deposit_amount)?;

    let allowance = deposit_args
        .read_allowance(owner, transaction_args.clone())
        .await?;
    if allowance >= deposit_amount {
        return Err(SubgraphError::InvalidAmount);
    }

    Ok(VaultCalldataResult(Bytes::copy_from_slice(
        &deposit_args.get_approve_calldata(transaction_args).await?,
    )))
}

/// Checks current ERC20 allowance for a vault.
///
/// Determines how much the orderbook contract is currently approved to spend
/// on behalf of the vault owner.
///
/// # Parameters
///
/// * `rpc_url` - Blockchain RPC endpoint
/// * `vault` - Vault to check allowance for
///
/// # Returns
///
/// * `Ok(VaultAllowanceResult)` - Current allowance amount as string
/// * `Err(SubgraphError)` - Network or contract errors
///
/// # Examples
///
/// ```javascript
/// const result = await checkVaultAllowance(
///   "https://polygon-rpc.com",
///   vault
/// );
/// if (result.error) {
///   console.error("Cannot check allowance:", result.error.readableMsg);
///   return;
/// }
/// const allowance = result.value;
/// // Do something with the allowance
/// ```
#[wasm_export(
    js_name = "checkVaultAllowance",
    unchecked_return_type = "VaultAllowanceResult"
)]
pub async fn check_vault_allowance(
    rpc_url: &str,
    vault: &SgVault,
) -> Result<VaultAllowanceResult, SubgraphError> {
    let (deposit_args, transaction_args) =
        get_deposit_and_transaction_args(rpc_url, vault, U256::ZERO)?;

    Ok(VaultAllowanceResult(
        deposit_args
            .read_allowance(Address::from_str(&vault.owner.0)?, transaction_args.clone())
            .await?,
    ))
}

pub fn validate_amount(amount: &str) -> Result<U256, SubgraphError> {
    let amount = U256::from_str(amount)?;
    if amount == U256::ZERO {
        return Err(SubgraphError::InvalidAmount);
    }
    Ok(amount)
}

pub fn validate_io_index(
    order: &SgOrder,
    is_input: bool,
    index: u8,
) -> Result<usize, SubgraphError> {
    let index = index as usize;
    if is_input {
        if order.inputs.len() <= index {
            return Err(SubgraphError::InvalidInputIndex);
        }
    } else if order.outputs.len() <= index {
        return Err(SubgraphError::InvalidOutputIndex);
    }
    Ok(index)
}

pub fn get_deposit_and_transaction_args(
    rpc_url: &str,
    vault: &SgVault,
    amount: U256,
) -> Result<(DepositArgs, TransactionArgs), SubgraphError> {
    let deposit_args = DepositArgs {
        token: Address::from_str(&vault.token.address.0)?,
        vault_id: U256::from_str(&vault.vault_id.0)?,
        amount,
    };
    let transaction_args = TransactionArgs {
        orderbook_address: Address::from_str(&vault.orderbook.id.0)?,
        rpc_url: rpc_url.to_string(),
        ..Default::default()
    };
    Ok((deposit_args, transaction_args))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::sol_types::SolCall;
    use rain_orderbook_subgraph_client::types::common::{SgBigInt, SgBytes, SgErc20, SgOrderbook};

    fn get_vault1() -> SgVault {
        SgVault {
            id: SgBytes("vault1".to_string()),
            owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
            vault_id: SgBigInt("0x10".to_string()),
            balance: SgBigInt("0x10".to_string()),
            token: SgErc20 {
                id: SgBytes("token1".to_string()),
                address: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                name: Some("Token 1".to_string()),
                symbol: Some("TKN1".to_string()),
                decimals: Some(SgBigInt("18".to_string())),
            },
            orderbook: SgOrderbook {
                id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
            },
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        }
    }
    #[cfg(target_family = "wasm")]
    mod wasm {
        use super::*;
        use rain_orderbook_bindings::IOrderBookV4::{deposit2Call, withdraw2Call};
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_get_vault_deposit_calldata() {
            let result = get_vault_deposit_calldata(&get_vault1(), "500")
                .await
                .unwrap();
            assert_eq!(
                result.0,
                Bytes::copy_from_slice(
                    &deposit2Call {
                        token: Address::from_str("0x0000000000000000000000000000000000000000")
                            .unwrap(),
                        vaultId: U256::from_str("0x10").unwrap(),
                        amount: U256::from_str("500").unwrap(),
                        tasks: vec![],
                    }
                    .abi_encode()
                )
            );

            let err = get_vault_deposit_calldata(&get_vault1(), "0")
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), SubgraphError::InvalidAmount.to_string());
        }

        #[wasm_bindgen_test]
        async fn test_get_vault_withdraw_calldata() {
            let result = get_vault_withdraw_calldata(&get_vault1(), "500")
                .await
                .unwrap();
            assert_eq!(
                result.0,
                Bytes::copy_from_slice(
                    &withdraw2Call {
                        token: Address::from_str("0x0000000000000000000000000000000000000000")
                            .unwrap(),
                        vaultId: U256::from_str("0x10").unwrap(),
                        targetAmount: U256::from_str("500").unwrap(),
                        tasks: vec![],
                    }
                    .abi_encode()
                )
            );

            let err = get_vault_withdraw_calldata(&get_vault1(), "0")
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), SubgraphError::InvalidAmount.to_string());
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use alloy_ethers_typecast::rpc::Response;
        use httpmock::MockServer;
        use rain_orderbook_bindings::IERC20::approveCall;
        use serde_json::{json, Value};

        fn get_vault1_json() -> Value {
            json!({
              "id": "vault1",
              "owner": "0x0000000000000000000000000000000000000000",
              "vaultId": "0x10",
              "balance": "0x10",
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
                "balance": "0x20",
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
        async fn test_get_vaults() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault1_json()]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault2_json()]
                    }
                }));
            });

            let result = get_vaults(
                vec![
                    MultiSubgraphArgs {
                        url: Url::parse(&sg_server.url("/sg1")).unwrap(),
                        name: "network-one".to_string(),
                    },
                    MultiSubgraphArgs {
                        url: Url::parse(&sg_server.url("/sg2")).unwrap(),
                        name: "network-two".to_string(),
                    },
                ],
                SgVaultsListFilterArgs {
                    owners: vec![],
                    hide_zero_balance: false,
                },
                SgPaginationArgs {
                    page: 1,
                    page_size: 10,
                },
            )
            .await
            .unwrap();
            assert_eq!(result.0.len(), 2);

            assert_eq!(result.0[0].subgraph_name, "network-one");
            let vault1 = result.0[0].vault.clone();
            assert_eq!(vault1.id.0, "vault1");
            assert_eq!(vault1.owner.0, "0x0000000000000000000000000000000000000000");
            assert_eq!(vault1.vault_id.0, "0x10");
            assert_eq!(vault1.balance.0, "0x10");
            assert_eq!(vault1.token.id.0, "token1");
            assert_eq!(
                vault1.orderbook.id.0,
                "0x0000000000000000000000000000000000000000"
            );

            assert_eq!(result.0[1].subgraph_name, "network-two");
            let vault2 = result.0[1].vault.clone();
            assert_eq!(vault2.id.0, "vault2");
            assert_eq!(vault2.owner.0, "0x0000000000000000000000000000000000000000");
            assert_eq!(vault2.vault_id.0, "0x20");
            assert_eq!(vault2.balance.0, "0x20");
            assert_eq!(vault2.token.id.0, "token2");
            assert_eq!(
                vault2.orderbook.id.0,
                "0x0000000000000000000000000000000000000000"
            );
        }

        #[tokio::test]
        async fn test_get_vault() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let vault = get_vault(&sg_server.url("/sg"), "vault1").await.unwrap();
            assert_eq!(vault.id.0, "vault1");
            assert_eq!(vault.owner.0, "0x0000000000000000000000000000000000000000");
            assert_eq!(vault.vault_id.0, "0x10");
            assert_eq!(vault.balance.0, "0x10");
            assert_eq!(vault.token.id.0, "token1");
            assert_eq!(
                vault.orderbook.id.0,
                "0x0000000000000000000000000000000000000000"
            );
            assert_eq!(vault.orders_as_output.len(), 0);
            assert_eq!(vault.orders_as_input.len(), 0);
            assert_eq!(vault.balance_changes.len(), 0);
        }

        #[tokio::test]
        async fn test_get_vault_balance_changes() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": [
                            {
                                "__typename": "Deposit",
                                "amount": "5000000000000000000",
                                "newVaultBalance": "5000000000000000000",
                                "oldVaultBalance": "0",
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

            let result = get_vault_balance_changes(
                &sg_server.url("/sg"),
                "vault1",
                SgPaginationArgs {
                    page: 1,
                    page_size: 1,
                },
            )
            .await
            .unwrap();
            assert_eq!(result.0.len(), 1);
            assert_eq!(result.0[0].__typename, "Deposit");
            assert_eq!(result.0[0].amount.0, "5000000000000000000");
            assert_eq!(result.0[0].new_vault_balance.0, "5000000000000000000");
            assert_eq!(result.0[0].old_vault_balance.0, "0");
            assert_eq!(
                result.0[0].vault.id.0,
                "0x166aeed725f0f3ef9fe62f2a9054035756d55e5560b17afa1ae439e9cd362902"
            );
            assert_eq!(result.0[0].vault.vault_id.0, "1");
            assert_eq!(
                result.0[0].vault.token.id.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                result.0[0].vault.token.address.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                result.0[0].vault.token.name,
                Some("Wrapped Flare".to_string())
            );
            assert_eq!(result.0[0].vault.token.symbol, Some("WFLR".to_string()));
            assert_eq!(
                result.0[0].vault.token.decimals,
                Some(SgBigInt("18".to_string()))
            );
            assert_eq!(result.0[0].timestamp.0, "1734054063");
            assert_eq!(
                result.0[0].transaction.id.0,
                "0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22"
            );
            assert_eq!(
                result.0[0].transaction.from.0,
                "0x7177b9d00bb5dbcaaf069cc63190902763783b09"
            );
            assert_eq!(result.0[0].transaction.block_number.0, "34407047");
            assert_eq!(result.0[0].transaction.timestamp.0, "1734054063");
            assert_eq!(
                result.0[0].orderbook.id.0,
                "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
            );
        }

        #[tokio::test]
        async fn test_get_vault_approval_calldata() {
            let rpc_server = MockServer::start_async().await;
            rpc_server.mock(|when, then| {
                when.path("/rpc");
                then.status(200).body(
                    Response::new_success(
                        1,
                        "0x0000000000000000000000000000000000000000000000000000000000000064",
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });

            let result = get_vault_approval_calldata(&rpc_server.url("/rpc"), &get_vault1(), "600")
                .await
                .unwrap();
            assert_eq!(
                result.0,
                Bytes::copy_from_slice(
                    &approveCall {
                        spender: Address::from_str("0x0000000000000000000000000000000000000000")
                            .unwrap(),
                        amount: U256::from(600),
                    }
                    .abi_encode(),
                )
            );

            let err = get_vault_approval_calldata(&rpc_server.url("/rpc"), &get_vault1(), "0")
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), SubgraphError::InvalidAmount.to_string());

            let err = get_vault_approval_calldata(&rpc_server.url("/rpc"), &get_vault1(), "90")
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), SubgraphError::InvalidAmount.to_string());

            let err = get_vault_approval_calldata(&rpc_server.url("/rpc"), &get_vault1(), "100")
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), SubgraphError::InvalidAmount.to_string());
        }

        #[tokio::test]
        async fn test_check_vault_allowance() {
            let rpc_server = MockServer::start_async().await;
            rpc_server.mock(|when, then| {
                when.path("/rpc");
                then.status(200).body(
                    Response::new_success(
                        1,
                        "0x0000000000000000000000000000000000000000000000000000000000000001",
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });

            let result = check_vault_allowance(&rpc_server.url("/rpc"), &get_vault1())
                .await
                .unwrap();
            assert_eq!(result.0, U256::from(1));
        }
    }
}
