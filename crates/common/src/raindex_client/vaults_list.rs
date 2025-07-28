#[cfg(not(target_family = "wasm"))]
use alloy::primitives::U256;
use alloy::{primitives::Bytes, sol_types::SolValue};
use rain_orderbook_bindings::OrderBook::multicallCall;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

use crate::raindex_client::vaults::RaindexVault;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct VaultsList(Vec<RaindexVault>);

impl VaultsList {
    pub fn new(vaults: Vec<RaindexVault>) -> Self {
        Self(vaults)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get_vaults(&self) -> &Vec<RaindexVault> {
        &self.0
    }

    pub fn get_withdrawable_vaults(&self) -> Vec<&RaindexVault> {
        self.0
            .iter()
            .filter(|vault| {
                #[cfg(target_family = "wasm")]
                {
                    // Use formatted_balance for checking non-zero balance in WASM
                    !vault.formatted_balance().is_empty() && vault.formatted_balance() != "0"
                }
                #[cfg(not(target_family = "wasm"))]
                {
                    vault.balance() > U256::ZERO
                }
            })
            .collect()
    }
    pub async fn get_withdraw_calldata(&self) -> Result<Bytes, VaultsListError> {
        let mut calldatas: Vec<Bytes> = Vec::new();
        let vaults_to_withdraw: Vec<&RaindexVault> = self.get_withdrawable_vaults();

        for vault in vaults_to_withdraw {
            let amount = {
                #[cfg(target_family = "wasm")]
                {
                    vault.formatted_balance()
                }
                #[cfg(not(target_family = "wasm"))]
                {
                    vault.balance().to_string()
                }
            };

            match vault.get_withdraw_calldata(amount).await {
                Ok(calldata) => calldatas.push(calldata),
                Err(e) => return Err(VaultsListError::WithdrawMulticallError(e.to_readable_msg())),
            }
        }

        let multicall = multicallCall { data: calldatas };
        let encoded = multicall.data.abi_encode();
        Ok(encoded.into())
    }
}

#[wasm_export]
impl VaultsList {
    /// Creates a new VaultsList from an array of RaindexVault objects
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const vaultsList = VaultsList.new([vault1, vault2, vault3]);
    /// ```
    #[wasm_export(
        js_name = "new",
        return_description = "A new VaultsList instance",
        preserve_js_class
    )]
    pub fn new_wasm(
        #[wasm_export(param_description = "Array of RaindexVault objects")] vaults: Vec<
            RaindexVault,
        >,
    ) -> Result<VaultsList, VaultsListError> {
        Ok(Self::new(vaults))
    }

    /// Creates a VaultsList from a JavaScript array using from() pattern
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const vaultsList = VaultsList.from([vault1, vault2, vault3]);
    /// ```
    #[wasm_export(
        js_name = "from",
        return_description = "A new VaultsList instance",
        preserve_js_class
    )]
    pub fn from_wasm(
        #[wasm_export(param_description = "Array of RaindexVault objects")] vaults: Vec<
            RaindexVault,
        >,
    ) -> Result<VaultsList, VaultsListError> {
        Ok(Self::new(vaults))
    }

    /// Returns the number of vaults in the list
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const count = vaultsList.length;
    /// console.log(`Found ${count} vaults`);
    /// ```
    #[wasm_export(
        js_name = "length",
        getter,
        return_description = "Number of vaults in the list",
        unchecked_return_type = "number"
    )]
    pub fn length_wasm(&self) -> Result<u32, VaultsListError> {
        Ok(self.len() as u32)
    }

    /// Checks if the vaults list is empty
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// if (vaultsList.isEmpty) {
    ///   console.log("No vaults found");
    /// }
    /// ```
    #[wasm_export(
        js_name = "isEmpty",
        getter,
        return_description = "True if the list is empty",
        unchecked_return_type = "boolean"
    )]
    pub fn is_empty_wasm(&self) -> Result<bool, VaultsListError> {
        Ok(self.is_empty())
    }

    /// Returns all vaults in the list
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const allVaults = vaultsList.getVaults();
    /// allVaults.forEach(vault => {
    ///   console.log(`Vault ID: ${vault.id}, Balance: ${vault.formattedBalance}`);
    /// });
    /// ```
    #[wasm_export(
        js_name = "getVaults",
        return_description = "Array of all vaults",
        unchecked_return_type = "RaindexVault[]",
        preserve_js_class
    )]
    pub fn get_vaults_wasm(&self) -> Result<Vec<RaindexVault>, VaultsListError> {
        Ok(self.get_vaults().clone())
    }

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
        unchecked_return_type = "Hex",
        preserve_js_class
    )]
    pub async fn get_withdraw_calldata_wasm(&self) -> Result<String, VaultsListError> {
        let calldata = self.get_withdraw_calldata().await?;
        Ok(calldata.to_string())
    }
}

#[derive(Error, Debug, Clone)]
pub enum VaultsListError {
    #[error("Failed to get withdraw multicall: {0}")]
    WithdrawMulticallError(String),
}

impl VaultsListError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            VaultsListError::WithdrawMulticallError(err) => {
                format!("Failed to generate withdraw multicall: {}", err)
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
              "balance": "0x00",
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
                "balance": "0x20",
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
            let vaults_list = VaultsList::new(get_vaults().await);
            assert_eq!(vaults_list.len(), 2);
        }

        #[tokio::test]
        async fn test_get_withdrawable_vaults() {
            let vaults_list = VaultsList::new(get_vaults().await);
            let withdrawable_vaults = vaults_list.get_withdrawable_vaults();
            assert_eq!(withdrawable_vaults.len(), 1);
            assert!(withdrawable_vaults[0].id().to_string() == "0x0234"); // vault2 has non-zero balance
        }

        #[tokio::test]
        async fn test_get_withdraw_calldata() {
            let vaults_list = VaultsList::new(get_vaults().await);

            let result = vaults_list.get_withdraw_calldata().await;
            let calldata = result.unwrap();
            assert!(!calldata.is_empty());
            assert!(calldata.len() > 2); // should contain vault2's ID
        }
    }

    #[test]
    fn test_vaults_list_new_empty() {
        let vaults_list = VaultsList::new(vec![]);

        assert_eq!(vaults_list.len(), 0);
        assert!(vaults_list.is_empty());
    }

    #[test]
    fn test_get_vaults_empty() {
        let vaults_list = VaultsList::new(vec![]);
        assert_eq!(vaults_list.len(), 0);
    }

    #[test]
    fn test_vaults_list_error_readable_msg() {
        let error = VaultsListError::WithdrawMulticallError("test error".to_string());
        let readable_msg = error.to_readable_msg();
        assert_eq!(
            readable_msg,
            "Failed to generate withdraw multicall: test error"
        );
    }

    #[test]
    fn test_vaults_list_error_display() {
        let error = VaultsListError::WithdrawMulticallError("test error".to_string());
        let display_msg = format!("{}", error);
        assert_eq!(display_msg, "Failed to get withdraw multicall: test error");
    }

    #[test]
    fn test_vaults_list_error_to_wasm_encoded_error() {
        let error = VaultsListError::WithdrawMulticallError("test error".to_string());
        let wasm_error: WasmEncodedError = error.into();
        assert_eq!(
            wasm_error.msg,
            "Failed to get withdraw multicall: test error"
        );
        assert_eq!(
            wasm_error.readable_msg,
            "Failed to generate withdraw multicall: test error"
        );
    }

    #[cfg(target_family = "wasm")]
    #[test]
    fn test_vaults_list_error_to_js_value() {
        let error = VaultsListError::WithdrawMulticallError("test error".to_string());
        let js_value: JsValue = error.into();
        // Just test that conversion doesn't panic
        assert!(!js_value.is_null());
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;

        #[tokio::test]
        async fn test_wasm_new_empty() {
            let result = VaultsList::new_wasm(vec![]);
            assert!(result.is_ok());

            let vaults_list = result.unwrap();
            assert_eq!(vaults_list.len(), 0);
        }

        #[tokio::test]
        async fn test_wasm_from_empty() {
            let result = VaultsList::from_wasm(vec![]);
            let vaults_list = result.unwrap();
            assert_eq!(vaults_list.len(), 0);
        }

        #[tokio::test]
        async fn test_wasm_length_empty() {
            let vaults_list = VaultsList::new(vec![]);
            let result = vaults_list.length_wasm();
            assert_eq!(result.unwrap(), 0);
        }

        #[tokio::test]
        async fn test_wasm_is_empty() {
            let empty_list = VaultsList::new(vec![]);
            let result = empty_list.is_empty_wasm();
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        #[tokio::test]
        async fn test_wasm_get_vaults_empty() {
            let vaults_list = VaultsList::new(vec![]);
            let result = vaults_list.get_vaults_wasm();
            assert!(result.is_ok());

            let retrieved_vaults = result.unwrap();
            assert_eq!(retrieved_vaults.len(), 0);
        }

        #[tokio::test]
        async fn test_wasm_get_withdrawable_vaults_empty() {
            let vaults_list = VaultsList::new(vec![]);
            let result = vaults_list.get_withdrawable_vaults_wasm();
            let withdrawable = result.unwrap();
            assert_eq!(withdrawable.len(), 0);
        }

        #[tokio::test]
        async fn test_wasm_get_withdraw_calldata_empty() {
            let vaults_list = VaultsList::new(vec![]);

            let result = vaults_list.get_withdraw_calldata_wasm().await;
            assert!(result.is_ok());

            // Should return valid calldata even for empty list
            let calldata = result.unwrap();
            assert!(!calldata.is_empty());
            assert!(calldata.starts_with("0x"));
        }
    }
}
