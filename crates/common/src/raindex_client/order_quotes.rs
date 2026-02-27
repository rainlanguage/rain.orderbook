use super::*;
use crate::raindex_client::orders::RaindexOrder;
use rain_math_float::Float;
use rain_orderbook_quote::{get_order_quotes, BatchOrderQuotesResponse, OrderQuoteValue, Pair};
use rain_orderbook_subgraph_client::utils::float::{F0, F1};
use std::ops::{Div, Mul};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RaindexOrderQuote {
    pub pair: Pair,
    pub block_number: u64,
    #[tsify(optional)]
    pub data: Option<RaindexOrderQuoteValue>,
    pub success: bool,
    #[tsify(optional)]
    pub error: Option<String>,
}
impl_wasm_traits!(RaindexOrderQuote);
impl RaindexOrderQuote {
    pub fn try_from_batch_order_quotes_response(
        value: BatchOrderQuotesResponse,
    ) -> Result<Self, RaindexError> {
        Ok(Self {
            pair: value.pair,
            block_number: value.block_number,
            data: value
                .data
                .map(RaindexOrderQuoteValue::try_from_order_quote_value)
                .transpose()?,
            success: value.success,
            error: value.error,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RaindexOrderQuoteValue {
    #[tsify(type = "Hex")]
    pub max_output: Float,
    pub formatted_max_output: String,
    #[tsify(type = "Hex")]
    pub max_input: Float,
    pub formatted_max_input: String,
    #[tsify(type = "Hex")]
    pub ratio: Float,
    pub formatted_ratio: String,
    #[tsify(type = "Hex")]
    pub inverse_ratio: Float,
    pub formatted_inverse_ratio: String,
}
impl_wasm_traits!(RaindexOrderQuoteValue);

impl RaindexOrderQuoteValue {
    pub fn try_from_order_quote_value(value: OrderQuoteValue) -> Result<Self, RaindexError> {
        let inverse_ratio = if F0.eq(value.ratio)? {
            F0
        } else {
            F1.div(value.ratio)?
        };

        let formatted_inverse_ratio = if F0.eq(value.ratio)? {
            "Infinity".to_string()
        } else {
            inverse_ratio.format()?
        };

        let max_input = value.max_output.mul(value.ratio)?;

        Ok(Self {
            max_output: value.max_output,
            formatted_max_output: value.max_output.format()?,
            max_input,
            formatted_max_input: max_input.format()?,
            ratio: value.ratio,
            formatted_ratio: value.ratio.format()?,
            inverse_ratio,
            formatted_inverse_ratio,
        })
    }
}

#[wasm_export]
impl RaindexOrder {
    /// Executes quotes directly from complete order objects without additional data fetching
    ///
    /// This function performs quote calculations using complete order data structures
    /// that typically come from previous subgraph queries. It generates quotes for all
    /// possible input/output token pairs within each order, providing comprehensive
    /// trading information without requiring additional network calls for order data.
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
        return_description = "List of batch quote responses with trading pair information",
        unchecked_return_type = "RaindexOrderQuote[]"
    )]
    pub async fn get_quotes(
        &self,
        #[wasm_export(
            js_name = "blockNumber",
            param_description = "Optional specific block number for historical quotes (uses latest if None)"
        )]
        block_number: Option<u64>,
        #[wasm_export(
            js_name = "chunkSize",
            param_description = "Optional quote chunk size override (defaults to 16)"
        )]
        chunk_size: Option<u32>,
    ) -> Result<Vec<RaindexOrderQuote>, RaindexError> {
        let rpcs = self.get_rpc_urls()?;
        let order_quotes = get_order_quotes(
            vec![self.clone().into_sg_order()?],
            block_number,
            rpcs.iter().map(|s| s.to_string()).collect(),
            chunk_size.map(|v| v as usize),
        )
        .await?;

        let mut result_order_quotes = vec![];
        for order_quote in order_quotes {
            let data = RaindexOrderQuote::try_from_batch_order_quotes_response(order_quote)?;
            result_order_quotes.push(data);
        }
        Ok(result_order_quotes)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod quote_non_wasm_tests {
        use super::*;
        use crate::local_db::OrderbookIdentifier;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use alloy::hex::encode_prefixed;
        use alloy::primitives::{b256, Address, U256};
        use alloy::{sol, sol_types::SolValue};
        use httpmock::MockServer;
        use rain_math_float::Float;
        use rain_orderbook_subgraph_client::utils::float::{F0_5, F2};
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
                  "balance": Float::parse("0.987".to_string()).unwrap(),
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
                  "balance": Float::parse("0.79799".to_string()).unwrap(),
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
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x1",
                }));
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
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": response_hex,
                }));
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
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                )
                .await
                .unwrap();
            let res = order.get_quotes(None, None).await.unwrap();
            assert_eq!(res.len(), 1);

            assert!(res[0].data.as_ref().unwrap().max_output.eq(F1).unwrap());

            assert!((res[0].data.as_ref().unwrap().ratio.eq(F2)).unwrap());

            assert!(res[0].success);
            assert_eq!(res[0].error, None);
            assert_eq!(res[0].pair.pair_name, "WFLR/sFLR");
            assert_eq!(res[0].pair.input_index, 0);
            assert_eq!(res[0].pair.output_index, 0);

            let res = res[0].clone();
            let data = res.data.unwrap();
            assert!(data.max_output.eq(F1).unwrap());
            assert_eq!(data.formatted_max_output, "1");
            assert!(data.max_input.eq(F2).unwrap());
            assert_eq!(data.formatted_max_input, "2");
            assert!(data.ratio.eq(F2).unwrap());
            assert_eq!(data.formatted_ratio, "2");
            assert!(data.inverse_ratio.eq(F0_5).unwrap());
            assert_eq!(data.formatted_inverse_ratio, "0.5");
            assert!(res.success);
            assert_eq!(res.error, None);
            assert_eq!(res.pair.pair_name, "WFLR/sFLR");
            assert_eq!(res.pair.input_index, 0);
            assert_eq!(res.pair.output_index, 0);
        }

        #[tokio::test]
        async fn test_get_order_quote_with_chunk_override() {
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
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x1",
                }));
            });

            let aggregate_result = vec![Result {
                success: true,
                returnData: quoteReturn {
                    exists: true,
                    outputMax: U256::from(1),
                    ioRatio: U256::from(2),
                }
                .abi_encode()
                .into(),
            }];
            let response_hex = encode_prefixed(aggregate_result.abi_encode());
            server.mock(|when, then| {
                when.path("/rpc");
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": response_hex,
                }));
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
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                )
                .await
                .unwrap();

            let res = order.get_quotes(None, Some(8)).await.unwrap();
            assert_eq!(res.len(), 1);
        }
    }
}
