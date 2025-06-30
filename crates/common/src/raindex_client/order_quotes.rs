use super::*;
use crate::raindex_client::orders::RaindexOrder;
use alloy::primitives::U256;
use rain_orderbook_quote::{get_order_quotes, BatchOrderQuotesResponse};

#[wasm_export]
impl RaindexOrder {
    /// Executes quotes directly from complete order objects without additional data fetching.
    ///
    /// This function performs quote calculations using complete order data structures
    /// that typically come from previous subgraph queries. It generates quotes for all
    /// possible input/output token pairs within each order, providing comprehensive
    /// trading information without requiring additional network calls for order data.
    ///
    /// ## Parameters
    ///
    /// - `blockNumber` - Optional specific block number for historical quotes (uses latest if None)
    /// - `gas` - Optional gas limit as string for quote simulations (uses default if None)
    ///
    /// ## Returns
    ///
    /// - `BatchOrderQuotesResponse[]` - Array of batch quote responses with trading pair information
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await getOrderQuote();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const quoteResponses = result.value;
    /// // Do something with the quoteResponses
    /// ```
    #[wasm_export(
        js_name = "getQuotes",
        unchecked_return_type = "BatchOrderQuotesResponse[]"
    )]
    pub async fn get_quotes(
        &self,
        #[wasm_export(js_name = "blockNumber")] block_number: Option<u64>,
        gas: Option<String>,
    ) -> Result<Vec<BatchOrderQuotesResponse>, RaindexError> {
        let gas_amount = gas.map(|v| U256::from_str(&v)).transpose()?;
        let rpc_url = self
            .get_raindex_client()?
            .get_rpc_url_for_chain(self.chain_id())?;
        let order_quotes = get_order_quotes(
            vec![self.clone().into_sg_order()?],
            block_number,
            rpc_url.to_string(),
            gas_amount,
        )
        .await?;
        Ok(order_quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod quote_non_wasm_tests {
        use super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use alloy::hex::encode_prefixed;
        use alloy::{sol, sol_types::SolValue};
        use alloy_ethers_typecast::rpc::Response;
        use httpmock::MockServer;
        use serde_json::{json, Value};

        sol!(
            struct Result {
                bool success;
                bytes returnData;
            }
        );
        sol!(
            struct quoteReturn {
                bool exists;
                uint256 outputMax;
                uint256 ioRatio;
            }
        );

        fn get_order1_json() -> Value {
            json!(                        {
              "id": "0x46891c626a8a188610b902ee4a0ce8a7e81915e1b922584f8168d14525899dfb",
              "orderBytes": "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000005f6c104ca9812ef91fe2e26a2e7187b92d3b0e800000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000022009cd210f509c66e18fab61fd30f76fb17c6c6cd09f0972ce0815b5b7630a1b050000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae600000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000075000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012f5bb1bfe104d351d99dcce1ccfb041ff244a2d3aaf83bd5c4f3fe20b3fceb372000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012f5bb1bfe104d351d99dcce1ccfb041ff244a2d3aaf83bd5c4f3fe20b3fceb372",
              "orderHash": "0x283508c8f56f4de2f21ee91749d64ec3948c16bc6b4bfe4f8d11e4e67d76f4e0",
              "owner": "0x0000000000000000000000000000000000000000",
              "outputs": [
                {
                  "id": "0x0000000000000000000000000000000000000000",
                  "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                  "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                  "balance": "987000000000000000",
                  "token": {
                    "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "name": "Staked FLR",
                    "symbol": "sFLR",
                    "decimals": "18"
                  },
                  "orderbook": {
                    "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                  },
                  "ordersAsOutput": [],
                  "ordersAsInput": [],
                  "balanceChanges": []
                }
              ],
              "inputs": [
                {
                  "id": "0x0000000000000000000000000000000000000000",
                  "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                  "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                  "balance": "797990000000000000",
                  "token": {
                    "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "name": "WFLR",
                    "symbol": "WFLR",
                    "decimals": "18"
                  },
                  "orderbook": {
                    "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                  },
                  "ordersAsOutput": [],
                  "ordersAsInput": [],
                  "balanceChanges": []
                },
              ],
              "orderbook": {
                "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
              },
              "active": true,
              "timestampAdded": "1739448802",
              "meta": null,
              "addEvents": [],
              "trades": [],
              "removeEvents": []
            })
        }

        #[tokio::test]
        async fn test_get_order_quote() {
            let server = MockServer::start_async().await;
            server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [get_order1_json()]
                    }
                }));
            });

            // block number 1
            server.mock(|when, then| {
                when.path("/rpc").body_contains("blockNumber");
                then.body(Response::new_success(1, "0x1").to_json_string().unwrap());
            });

            let aggreate_result = vec![Result {
                success: true,
                returnData: quoteReturn {
                    exists: true,
                    outputMax: U256::from(1),
                    ioRatio: U256::from(2),
                }
                .abi_encode()
                .into(),
            }];
            let response_hex = encode_prefixed(aggreate_result.abi_encode());
            server.mock(|when, then| {
                when.path("/rpc");
                then.body(
                    Response::new_success(1, &response_hex)
                        .to_json_string()
                        .unwrap(),
                );
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &server.url("/sg"),
                    "http://localhost:3000",
                    &server.url("/rpc"),
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();
            let order = raindex_client
                .get_order_by_hash(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "0x1".to_string(),
                )
                .await
                .unwrap();
            let res = order.get_quotes(None, None).await.unwrap();
            assert_eq!(res.len(), 1);
            assert_eq!(res[0].data.unwrap().max_output, U256::from(1));
            assert_eq!(res[0].data.unwrap().ratio, U256::from(2));
            assert!(res[0].success);
            assert_eq!(res[0].error, None);
            assert_eq!(res[0].pair.pair_name, "WFLR/sFLR");
            assert_eq!(res[0].pair.input_index, 0);
            assert_eq!(res[0].pair.output_index, 0);
        }

        #[tokio::test]
        async fn test_get_order_quote_invalid_values() {
            let server = MockServer::start_async().await;
            server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [get_order1_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();
            let order = raindex_client
                .get_order_by_hash(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "0x1".to_string(),
                )
                .await
                .unwrap();

            let err = order
                .get_quotes(None, Some("invalid-gas".to_string()))
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), "digit 18 is out of range for base 10");
            assert_eq!(
                err.to_readable_msg(),
                "Invalid number format: digit 18 is out of range for base 10. Please provide a valid numeric value."
            );
        }
    }
}
