use alloy::{primitives::Bytes, sol_types::SolCall};
use rain_math_float::Float;
use rain_orderbook_bindings::OrderBook::multicallCall;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

use crate::raindex_client::vaults::RaindexVault;
use once_cell::sync::Lazy;

static ZERO_FLOAT: Lazy<Float> = Lazy::new(|| Float::parse("0".to_string()).unwrap());

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct RaindexVaultsList(Vec<RaindexVault>);

impl RaindexVaultsList {
    pub fn new(vaults: Vec<RaindexVault>) -> Self {
        Self(vaults)
    }
    pub fn get_withdrawable_vaults(&self) -> Vec<&RaindexVault> {
        self.0
            .iter()
            .filter(|vault| vault.balance().gt(*ZERO_FLOAT).unwrap_or(false))
            .collect()
    }
    pub async fn get_withdraw_calldata(&self) -> Result<Bytes, VaultsListError> {
        let mut calldatas: Vec<Bytes> = Vec::new();
        let vaults_to_withdraw = self.get_withdrawable_vaults();
        // If no vaults to withdraw, return error
        if vaults_to_withdraw.is_empty() {
            return Err(VaultsListError::NoWithdrawableVaults);
        }
        let mut orderbook_id_iter = vaults_to_withdraw.iter().map(|v| v.orderbook());
        let first_orderbook_id = orderbook_id_iter.next();
        if let Some(first_id) = first_orderbook_id {
            if orderbook_id_iter.any(|id| id != first_id) {
                return Err(VaultsListError::MultipleOrderbooksUsed);
            }
        }
        // Generate multicall calldata for all vaults
        for vault in vaults_to_withdraw {
            let amount = vault.balance().format().map_err(|e| {
                VaultsListError::WithdrawMulticallError(format!("Failed to format balance: {}", e))
            })?;
            match vault.get_withdraw_calldata(amount.clone()).await {
                Ok(calldata) => calldatas.push(calldata),
                Err(e) => return Err(VaultsListError::WithdrawMulticallError(e.to_readable_msg())),
            }
        }

        if calldatas.len() == 1 {
            // If only one vault, return its calldata directly
            return Ok(calldatas[0].clone());
        }
        // Otherwise, create a multicall with all vaults' calldata
        let multicall = multicallCall { data: calldatas };
        let encoded = multicall.abi_encode();
        Ok(encoded.into())
    }
}

#[cfg(not(target_family = "wasm"))]
impl RaindexVaultsList {
    pub fn items(&self) -> Vec<RaindexVault> {
        self.0.clone()
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVaultsList {
    /// Returns all vaults in the list
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const allVaults = vaultsList.items;
    /// allVaults.forEach(vault => {
    ///   console.log(`Vault ID: ${vault.id}, Balance: ${vault.formattedBalance}`);
    /// });
    /// ```
    #[wasm_bindgen(getter)]
    pub fn items(&self) -> Vec<RaindexVault> {
        self.0.clone()
    }
}

#[wasm_export]
impl RaindexVaultsList {
    /// Returns only vaults that have a balance greater than zero
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const withdrawableVaults = vaultsList.getWithdrawableVaults();
    /// console.log(`${withdrawableVaults.length} vaults can be withdrawn from`);
    /// ```
    #[wasm_export(
        js_name = "getWithdrawableVaults",
        return_description = "Array of vaults with non-zero balance",
        unchecked_return_type = "RaindexVault[]",
        preserve_js_class
    )]
    pub fn get_withdrawable_vaults_wasm(&self) -> Result<Vec<RaindexVault>, VaultsListError> {
        Ok(self
            .get_withdrawable_vaults()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Generates multicall withdraw calldata for all withdrawable vaults
    ///
    /// Creates a single transaction that can withdraw from multiple vaults
    /// at once, optimizing gas costs and transaction efficiency.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vaultsList.getWithdrawCalldata();
    /// if (result.error) {
    ///   console.error("Error generating calldata:", result.error.readableMsg);
    ///   return;
    /// }
    /// const calldata = result.value;
    /// // Use calldata for transaction
    /// ```
    #[wasm_export(
        js_name = "getWithdrawCalldata",
        return_description = "Encoded multicall calldata for withdrawing from all vaults",
        unchecked_return_type = "Hex"
    )]
    pub async fn get_withdraw_calldata_wasm(&self) -> Result<Bytes, VaultsListError> {
        let calldata = self.get_withdraw_calldata().await?;
        Ok(calldata)
    }
}

#[derive(Error, Debug, Clone)]
pub enum VaultsListError {
    #[error("Failed to get withdraw multicall: {0}")]
    WithdrawMulticallError(String),
    #[error("No withdrawable vaults available")]
    NoWithdrawableVaults,
    #[error("All vaults must share the same orderbook for batch withdrawal")]
    MultipleOrderbooksUsed,
}

impl VaultsListError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            VaultsListError::WithdrawMulticallError(err) => {
                format!("Failed to generate withdraw multicall: {}", err)
            }
            VaultsListError::NoWithdrawableVaults => "No withdrawable vaults available".to_string(),
            VaultsListError::MultipleOrderbooksUsed => {
                "All vaults must share the same orderbook for batch withdrawal".to_string()
            }
        }
    }
}

impl From<VaultsListError> for JsValue {
    fn from(value: VaultsListError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<VaultsListError> for WasmEncodedError {
    fn from(value: VaultsListError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use crate::raindex_client::{tests::get_test_yaml, RaindexClient};
        use httpmock::MockServer;
        use serde_json::{json, Value};

        fn get_vault1_json() -> Value {
            json!({
              "id": "0x0123",
              "owner": "0x0000000000000000000000000000000000000000",
              "vaultId": "0x10",
              "balance": "0x0000000000000000000000000000000000000000000000000000000000000000",
              "token": {
                "id": "token1",
                "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
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
                "id": "0x0234",
                "owner": "0x0000000000000000000000000000000000000000",
                "vaultId": "0x20",
                "balance": "0x0000000000000000000000000000000000000000000000000000000000000020",
                "token": {
                    "id": "token2",
                    "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
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

        async fn get_vaults() -> Vec<RaindexVault> {
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

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let vaults = raindex_client.get_vaults(None, None, None).await.unwrap();
            vaults
        }

        #[tokio::test]
        async fn test_get_vaults_not_empty() {
            let vaults_list = RaindexVaultsList::new(get_vaults().await);
            assert_eq!(vaults_list.0.len(), 2);
        }

        #[tokio::test]
        async fn test_get_withdrawable_vaults() {
            let vaults_list = RaindexVaultsList::new(get_vaults().await);
            let withdrawable_vaults = vaults_list.get_withdrawable_vaults();
            assert_eq!(withdrawable_vaults.len(), 1);
            assert_eq!(withdrawable_vaults[0].id().to_string(), "0x0234"); // vault2 has non-zero balance
        }

        #[tokio::test]
        async fn test_get_withdraw_calldata() {
            let vaults_list = RaindexVaultsList::new(get_vaults().await);

            let result = vaults_list.get_withdraw_calldata().await;
            let calldata = result.unwrap();
            assert!(!calldata.is_empty());
            assert!(calldata.len() > 2); // should contain vault2's ID
        }
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_wasm_get_vaults() {
            let vaults_list = RaindexVaultsList::new(vec![]);
            let vaults = vaults_list.items();
            assert_eq!(vaults.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_wasm_get_withdrawable_vaults_empty() {
            let vaults_list = RaindexVaultsList::new(vec![]);
            let result = vaults_list.get_withdrawable_vaults_wasm();
            let withdrawable = result.unwrap();
            assert_eq!(withdrawable.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_wasm_get_withdraw_calldata_empty() {
            let vaults_list = RaindexVaultsList::new(vec![]);
            let result = vaults_list.get_withdraw_calldata_wasm().await;
            assert!(result.is_err());
        }
    }
}
