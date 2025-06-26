use std::str::FromStr;

use super::*;
use alloy::primitives::{Address, U256};
use rain_orderbook_subgraph_client::types::{common::SgTransaction, Id};
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexTransaction {
    id: String,
    from: Address,
    block_number: U256,
    timestamp: U256,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexTransaction {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn from(&self) -> String {
        self.from.to_string()
    }
    #[wasm_bindgen(getter = blockNumber)]
    pub fn block_number(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.block_number.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.timestamp.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexTransaction {
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn from(&self) -> Address {
        self.from
    }
    pub fn block_number(&self) -> U256 {
        self.block_number
    }
    pub fn timestamp(&self) -> U256 {
        self.timestamp
    }
}

#[wasm_export]
impl RaindexClient {
    /// Fetches transaction details for a given transaction hash.on a given chain
    ///
    /// Retrieves basic transaction information including sender, block number,
    /// and timestamp.
    ///
    /// ## Parameters
    ///
    /// * `chain_id` - Chain ID for the network
    /// * `tx_hash` - Transaction hash
    ///
    /// ## Returns
    ///
    /// * `RaindexTransaction` - Transaction details
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await getTransaction(
    ///   1,
    ///   "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    /// );
    /// if (result.error) {
    ///   console.error("Transaction not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const transaction = result.value;
    /// // Do something with the transaction
    /// ```
    #[wasm_export(
        js_name = "getTransaction",
        unchecked_return_type = "RaindexTransaction"
    )]
    pub async fn get_transaction(
        &self,
        chain_id: u64,
        orderbook_address: String,
        tx_hash: String,
    ) -> Result<RaindexTransaction, RaindexError> {
        let client = self.get_orderbook_client(chain_id, orderbook_address)?;
        let transaction = client.transaction_detail(Id::new(tx_hash)).await?;
        transaction.try_into()
    }
}

impl TryFrom<SgTransaction> for RaindexTransaction {
    type Error = RaindexError;
    fn try_from(transaction: SgTransaction) -> Result<Self, Self::Error> {
        Ok(Self {
            id: transaction.id.0,
            from: Address::from_str(&transaction.from.0)?,
            block_number: U256::from_str(&transaction.block_number.0)?,
            timestamp: U256::from_str(&transaction.timestamp.0)?,
        })
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use httpmock::MockServer;
        use serde_json::json;

        #[tokio::test]
        async fn test_get_transaction() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "transaction": {
                            "id": "tx1",
                            "from": "0x1000000000000000000000000000000000000000",
                            "blockNumber": "12345",
                            "timestamp": "1734054063"
                        }
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();
            let tx = raindex_client
                .get_transaction(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "0x123".to_string(),
                )
                .await
                .unwrap();
            assert_eq!(tx.id(), "tx1".to_string());
            assert_eq!(
                tx.from(),
                Address::from_str("0x1000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(tx.block_number(), U256::from_str("12345").unwrap());
            assert_eq!(tx.timestamp(), U256::from_str("1734054063").unwrap());
        }
    }
}
