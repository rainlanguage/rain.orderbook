use crate::{error::Error, BatchQuoteSpec, QuoteSpec};
use crate::{
    get_order_quotes, BatchOrderQuotesResponse, BatchQuoteTarget, OrderQuoteValue, QuoteTarget,
};
use alloy::hex::FromHexError;
use alloy::primitives::ruint::ParseError;
use alloy::primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_subgraph_client::{types::common::SgOrder, utils::make_order_id};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(untagged)]
pub enum QuoteResultEnum {
    Success {
        value: OrderQuoteValue,
        #[tsify(type = "undefined")]
        error: Option<String>,
    },
    Err {
        #[tsify(type = "undefined")]
        value: Option<OrderQuoteValue>,
        error: String,
    },
}
impl_wasm_traits!(QuoteResultEnum);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DoQuoteTargetsResult(pub Vec<QuoteResultEnum>);
impl_wasm_traits!(DoQuoteTargetsResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DoQuoteSpecsResult(pub Vec<QuoteResultEnum>);
impl_wasm_traits!(DoQuoteSpecsResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTargetResult(
    #[tsify(type = "(QuoteTarget | undefined)[]")] pub Vec<Option<QuoteTarget>>,
);
impl_wasm_traits!(QuoteTargetResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DoOrderQuoteResult(pub Vec<BatchOrderQuotesResponse>);
impl_wasm_traits!(DoOrderQuoteResult);

/// Get subgraph represented "order_id" of a QuoteTarget
#[wasm_export(js_name = "getId", unchecked_return_type = "string")]
pub fn get_id(orderbook: &str, order_hash: &str) -> Result<String, QuoteBindingsError> {
    let orderbook = Address::from_hex(orderbook)?;
    let order_hash = U256::from_str(order_hash)?;
    Ok(encode_prefixed(make_order_id(orderbook, order_hash)))
}

/// Quotes the target on the given rpc url
/// Resolves with array of OrderQuoteValue object or a string error
#[wasm_export(
    js_name = "doQuoteTargets",
    unchecked_return_type = "DoQuoteTargetsResult"
)]
pub async fn do_quote_targets(
    quote_targets: BatchQuoteTarget,
    rpc_url: String,
    block_number: Option<u64>,
    gas: Option<String>,
    multicall_address: Option<String>,
) -> Result<DoQuoteTargetsResult, QuoteBindingsError> {
    let multicall_address = multicall_address
        .map(|v| Address::from_hex(v))
        .transpose()?;
    let gas_value = gas.map(|v| U256::from_str(&v)).transpose()?;
    let quote_targets: Vec<QuoteTarget> = quote_targets
        .0
        .iter()
        .map(|v| QuoteTarget::from(v.clone()))
        .collect();
    let batch_quote_target = BatchQuoteTarget(quote_targets);

    let quotes = batch_quote_target
        .do_quote(&rpc_url, block_number, gas_value, multicall_address)
        .await?;

    let res = quotes
        .into_iter()
        .map(|q| match q {
            Ok(v) => QuoteResultEnum::Success {
                value: v,
                error: None,
            },
            Err(e) => QuoteResultEnum::Err {
                value: None,
                error: e.to_string(),
            },
        })
        .collect();

    Ok(DoQuoteTargetsResult(res))
}

/// Given a subgraph url, will fetch the order details from the subgraph and
/// then quotes them using the given rpc url.
/// Resolves with array of OrderQuoteValue object or a string error
#[wasm_export(js_name = "doQuoteSpecs", unchecked_return_type = "DoQuoteSpecsResult")]
pub async fn do_quote_specs(
    quote_specs: BatchQuoteSpec,
    subgraph_url: String,
    rpc_url: String,
    block_number: Option<u64>,
    gas: Option<String>,
    multicall_address: Option<String>,
) -> Result<DoQuoteSpecsResult, QuoteBindingsError> {
    let multicall_address = multicall_address
        .map(|v| Address::from_hex(v))
        .transpose()?;
    let gas_value = gas.map(|v| U256::from_str(&v)).transpose()?;
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);

    let quotes = batch_quote_spec
        .do_quote(
            &subgraph_url,
            &rpc_url,
            block_number,
            gas_value,
            multicall_address,
        )
        .await?;

    let res = quotes
        .into_iter()
        .map(|q| match q {
            Ok(v) => QuoteResultEnum::Success {
                value: v,
                error: None,
            },
            Err(e) => QuoteResultEnum::Err {
                value: None,
                error: e.to_string(),
            },
        })
        .collect();

    Ok(DoQuoteSpecsResult(res))
}

/// Given a subgraph url, will fetch orders details and returns their
/// respective quote targets.
/// Resolves with array of QuoteTarget object or undefined if no result
/// found on subgraph for a specific spec
#[wasm_export(
    js_name = "getQuoteTargetFromSubgraph",
    unchecked_return_type = "QuoteTargetResult"
)]
pub async fn get_batch_quote_target_from_subgraph(
    quote_specs: BatchQuoteSpec,
    subgraph_url: String,
) -> Result<QuoteTargetResult, QuoteBindingsError> {
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);

    let quote_targets = batch_quote_spec
        .get_batch_quote_target_from_subgraph(&subgraph_url)
        .await?;
    Ok(QuoteTargetResult(quote_targets))
}

/// Get the quote for an order
/// Resolves with a BatchOrderQuotesResponse object
#[wasm_export(
    js_name = "getOrderQuote",
    unchecked_return_type = "DoOrderQuoteResult"
)]
pub async fn get_order_quote(
    order: Vec<SgOrder>,
    rpc_url: String,
    block_number: Option<u64>,
    gas: Option<String>,
) -> Result<DoOrderQuoteResult, QuoteBindingsError> {
    let gas_value = gas.map(|v| U256::from_str(&v)).transpose()?;
    let order_quotes = get_order_quotes(order, block_number, rpc_url, gas_value).await?;
    Ok(DoOrderQuoteResult(order_quotes))
}

#[derive(Error, Debug)]
pub enum QuoteBindingsError {
    #[error(transparent)]
    QuoteError(#[from] Error),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    U256ParseError(#[from] ParseError),
    #[error("JavaScript error: {0}")]
    JsError(String),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}

impl QuoteBindingsError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::QuoteError(e) => format!("Failed to get quote: {}", e),
            Self::FromHexError(e) => format!("Invalid address format: {}", e),
            Self::U256ParseError(e) => format!("Invalid numeric value: {}", e),
            Self::JsError(msg) => format!("Internal JavaScript error: {}", msg),
            Self::SerdeWasmBindgenError(err) => {
                format!("Failed to serialize/deserialize data: {}", err)
            }
        }
    }
}

impl From<QuoteBindingsError> for JsValue {
    fn from(value: QuoteBindingsError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<QuoteBindingsError> for WasmEncodedError {
    fn from(value: QuoteBindingsError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_get_id() {
            let orderbook =
                Address::from_str("0x0123456789123456789123456789123456789123").unwrap();
            let order_hash = U256::from(30);
            let expected_id = encode_prefixed(make_order_id(orderbook, order_hash));

            let res = get_id(&orderbook.to_string(), &order_hash.to_string()).unwrap();
            assert_eq!(res, expected_id);

            let err = get_id("invalid-hex", &order_hash.to_string()).unwrap_err();
            assert_eq!(err.to_string(), "Odd number of digits");
            assert_eq!(
                err.to_readable_msg(),
                "Failed to parse orderbook address: Odd number of digits"
            );

            let err = get_id(&orderbook.to_string(), "invalid-hash").unwrap_err();
            assert_eq!(err.to_string(), "digit 18 is out of range for base 10");
            assert_eq!(
                err.to_readable_msg(),
                "Failed to parse u256 value: digit 18 is out of range for base 10"
            );
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod quote_non_wasm_tests {
        use super::*;
        use crate::QuoteResult;
        use alloy::hex;
        use alloy::primitives::{Bytes, FixedBytes};
        use alloy::{sol, sol_types::SolValue};
        use alloy_ethers_typecast::rpc::Response;
        use httpmock::MockServer;
        use rain_orderbook_bindings::IOrderBookV4::{EvaluableV3, OrderV3, Quote, IO};
        use rain_orderbook_subgraph_client::types::common::{
            SgAddOrder, SgBigInt, SgBytes, SgErc20, SgOrderbook, SgTransaction, SgVault,
        };
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

        fn get_quote_config() -> Quote {
            Quote {
                order: OrderV3 {
                    owner: Address::from_str("0x2000000000000000000000000000000000000000").unwrap(),
                    evaluable: EvaluableV3 {
                        interpreter: Address::from_str(
                            "0x0000000000000000000000000000000000000001",
                        )
                        .unwrap(),
                        store: Address::from_str("0x0000000000000000000000000000000000000002")
                            .unwrap(),
                        bytecode: Bytes::from_str("0x").unwrap(),
                    },
                    validInputs: vec![IO {
                        token: Address::from_str("0x0000000000000000000000000000000000000001")
                            .unwrap(),
                        decimals: 6,
                        vaultId: U256::from(20),
                    }],
                    validOutputs: vec![IO {
                        token: Address::from_str("0x0000000000000000000000000000000000000002")
                            .unwrap(),
                        decimals: 18,
                        vaultId: U256::from(100),
                    }],
                    nonce: FixedBytes::from_str(
                        "0x1230000000000000000000000000000000000000000000000000000000000000",
                    )
                    .unwrap(),
                },
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            }
        }

        fn get_batch_quote_targets() -> BatchQuoteTarget {
            BatchQuoteTarget(vec![QuoteTarget {
                orderbook: Address::from_str("0x1000000000000000000000000000000000000000").unwrap(),
                quote_config: get_quote_config(),
            }])
        }

        fn get_batch_quote_specs() -> BatchQuoteSpec {
            BatchQuoteSpec(vec![
                QuoteSpec {
                    order_hash: U256::from(30),
                    input_io_index: 0,
                    output_io_index: 0,
                    orderbook: Address::from_str("0x1000000000000000000000000000000000000000")
                        .unwrap(),
                    signed_context: vec![],
                },
                QuoteSpec {
                    order_hash: U256::from(30),
                    input_io_index: 1,
                    output_io_index: 1,
                    orderbook: Address::from_str("0x2000000000000000000000000000000000000000")
                        .unwrap(),
                    signed_context: vec![],
                },
            ])
        }

        fn get_order_json() -> Value {
            json!({
                "data": {
                    "orders": [
                        {
                            "id": make_order_id(Address::from_str("0x1000000000000000000000000000000000000000").unwrap(), U256::from(30)),
                            "orderBytes":
                                "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a01230000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000064",
                            "orderHash": "0x8a3fbb9caf53f18f1f78d90c48dbe4612bcd93285ed0fc033009b4a96ea2aaed",
                            "owner": "0x0000000000000000000000000000000000000000",
                            "outputs": [
                                {
                                    "id": "0x0000000000000000000000000000000000000000",
                                    "token": {
                                        "id": "0x0000000000000000000000000000000000000000",
                                        "address": "0x0000000000000000000000000000000000000000",
                                        "name": "T1",
                                        "symbol": "T1",
                                        "decimals": "0"
                                    },
                                    "balance": "0",
                                    "vaultId": "0",
                                    "owner": "0x0000000000000000000000000000000000000000",
                                    "ordersAsOutput": [],
                                    "ordersAsInput": [],
                                    "balanceChanges": [],
                                    "orderbook": {
                                        "id": "0x0000000000000000000000000000000000000000"
                                    }
                                }
                            ],
                            "inputs": [
                                {
                                    "id": "0x0000000000000000000000000000000000000000",
                                    "token": {
                                        "id": "0x0000000000000000000000000000000000000000",
                                        "address": "0x0000000000000000000000000000000000000000",
                                        "name": "T2",
                                        "symbol": "T2",
                                        "decimals": "0"
                                    },
                                    "balance": "0",
                                    "vaultId": "0",
                                    "owner": "0x0000000000000000000000000000000000000000",
                                    "ordersAsOutput": [],
                                    "ordersAsInput": [],
                                    "balanceChanges": [],
                                    "orderbook": {
                                        "id": "0x0000000000000000000000000000000000000000"
                                    }
                                }
                            ],
                            "active": true,
                            "addEvents": [
                                {
                                    "transaction": {
                                        "blockNumber": "0",
                                        "timestamp": "0",
                                        "id": "0x0000000000000000000000000000000000000000",
                                        "from": "0x0000000000000000000000000000000000000000"
                                    }
                                }
                            ],
                            "meta": null,
                            "timestampAdded": "0",
                            "orderbook": {
                                "id": "0x0000000000000000000000000000000000000000"
                            },
                            "trades": [],
                            "removeEvents": []
                        },
                        {
                            "id": make_order_id(Address::from_str("0x2000000000000000000000000000000000000000").unwrap(), U256::from(30)),
                            "orderBytes":
                                "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a01230000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000064",
                            "orderHash": "0x8a3fbb9caf53f18f1f78d90c48dbe4612bcd93285ed0fc033009b4a96ea2aaed",
                            "owner": "0x0000000000000000000000000000000000000000",
                            "outputs": [
                                {
                                    "id": "0x0000000000000000000000000000000000000000",
                                    "token": {
                                        "id": "0x0000000000000000000000000000000000000000",
                                        "address": "0x0000000000000000000000000000000000000000",
                                        "name": "T1",
                                        "symbol": "T1",
                                        "decimals": "0"
                                    },
                                    "balance": "0",
                                    "vaultId": "0",
                                    "owner": "0x0000000000000000000000000000000000000000",
                                    "ordersAsOutput": [],
                                    "ordersAsInput": [],
                                    "balanceChanges": [],
                                    "orderbook": {
                                        "id": "0x0000000000000000000000000000000000000000"
                                    }
                                }
                            ],
                            "inputs": [
                                {
                                    "id": "0x0000000000000000000000000000000000000000",
                                    "token": {
                                        "id": "0x0000000000000000000000000000000000000000",
                                        "address": "0x0000000000000000000000000000000000000000",
                                        "name": "T2",
                                        "symbol": "T2",
                                        "decimals": "0"
                                    },
                                    "balance": "0",
                                    "vaultId": "0",
                                    "owner": "0x0000000000000000000000000000000000000000",
                                    "ordersAsOutput": [],
                                    "ordersAsInput": [],
                                    "balanceChanges": [],
                                    "orderbook": {
                                        "id": "0x0000000000000000000000000000000000000000"
                                    }
                                }
                            ],
                            "active": true,
                            "addEvents": [
                                {
                                    "transaction": {
                                        "blockNumber": "0",
                                        "timestamp": "0",
                                        "id": "0x0000000000000000000000000000000000000000",
                                        "from": "0x0000000000000000000000000000000000000000"
                                    }
                                }
                            ],
                            "meta": null,
                            "timestampAdded": "0",
                            "orderbook": {
                                "id": "0x0000000000000000000000000000000000000000"
                            },
                            "trades": [],
                            "removeEvents": []
                        },
                    ]
                }
            })
        }

        #[tokio::test]
        async fn test_do_quote_targets() {
            let rpc_server = MockServer::start_async().await;

            let aggreate_result = vec![
                Result {
                    success: true,
                    returnData: quoteReturn {
                        exists: true,
                        outputMax: U256::from(1),
                        ioRatio: U256::from(2),
                    }
                    .abi_encode()
                    .into(),
                },
                Result {
                    success: false,
                    returnData: Bytes::from_str("0x123abcdf").unwrap(),
                },
            ];
            let response_hex = encode_prefixed(aggreate_result.abi_encode());

            rpc_server.mock(|when, then| {
                when.path("/rpc");
                then.body(
                    Response::new_success(1, &response_hex)
                        .to_json_string()
                        .unwrap(),
                );
            });

            let res = do_quote_targets(
                get_batch_quote_targets(),
                rpc_server.url("/rpc"),
                None,
                None,
                None,
            )
            .await
            .unwrap();

            assert_eq!(res.0.len(), 2);
            match &res.0[0] {
                QuoteResultEnum::Success { value, error } => {
                    assert!(error.is_none());
                    assert_eq!(value.max_output, U256::from(1));
                    assert_eq!(value.ratio, U256::from(2));
                }
                QuoteResultEnum::Err { .. } => {
                    panic!("Expected success, got error");
                }
            }
            match &res.0[1] {
                QuoteResultEnum::Success { .. } => {
                    panic!("Expected error, got success");
                }
                QuoteResultEnum::Err { value, error } => {
                    assert!(value.is_none());
                    assert_eq!(
                        error,
                        "Execution reverted with unknown error. Data: \"123abcdf\" "
                    );
                }
            }
        }

        #[tokio::test]
        async fn test_do_quote_targets_invalid_values() {
            let err = do_quote_targets(
                get_batch_quote_targets(),
                "some-url".to_string(),
                None,
                None,
                Some("invalid-address".to_string()),
            )
            .await
            .unwrap_err();
            assert_eq!(err.to_string(), "Odd number of digits");
            assert_eq!(
                err.to_readable_msg(),
                "Invalid address format: Odd number of digits"
            );

            let err = do_quote_targets(
                get_batch_quote_targets(),
                "some-url".to_string(),
                None,
                Some("invalid-gas".to_string()),
                None,
            )
            .await
            .unwrap_err();
            assert_eq!(err.to_string(), "digit 18 is out of range for base 10");
            assert_eq!(
                err.to_readable_msg(),
                "Invalid numeric value: digit 18 is out of range for base 10"
            );
        }

        #[tokio::test]
        async fn test_do_quote_specs() {
            let subgraph_server = MockServer::start_async().await;
            let rpc_server = MockServer::start_async().await;

            subgraph_server.mock(|when, then| {
                when.path("/subgraph");
                then.json_body_obj(&get_order_json());
            });

            let aggreate_result = vec![
                Result {
                    success: true,
                    returnData: quoteReturn {
                        exists: true,
                        outputMax: U256::from(1),
                        ioRatio: U256::from(2),
                    }
                    .abi_encode()
                    .into(),
                },
                Result {
                    success: false,
                    returnData: Bytes::from_str("0x123abcdf").unwrap(),
                },
            ];
            rpc_server.mock(|when, then| {
                when.path("/rpc");
                then.body(
                    Response::new_success(1, &encode_prefixed(aggreate_result.abi_encode()))
                        .to_json_string()
                        .unwrap(),
                );
            });

            let res = do_quote_specs(
                get_batch_quote_specs(),
                subgraph_server.url("/subgraph"),
                rpc_server.url("/rpc"),
                None,
                None,
                None,
            )
            .await
            .unwrap();

            assert_eq!(res.0.len(), 2);
            match &res.0[0] {
                QuoteResultEnum::Success { value, error } => {
                    assert!(error.is_none());
                    assert_eq!(value.max_output, U256::from(1));
                    assert_eq!(value.ratio, U256::from(2));
                }
                QuoteResultEnum::Err { error, .. } => {
                    panic!("Expected success, got error: {}", error);
                }
            }
            match &res.0[1] {
                QuoteResultEnum::Success { .. } => {
                    panic!("Expected error, got success");
                }
                QuoteResultEnum::Err { value, error } => {
                    assert!(value.is_none());
                    assert_eq!(
                        error,
                        "Execution reverted with unknown error. Data: \"123abcdf\" "
                    );
                }
            }
        }

        #[tokio::test]
        async fn test_do_quote_specs_invalid_values() {
            let err = do_quote_specs(
                get_batch_quote_specs(),
                "some-url".to_string(),
                "some-url".to_string(),
                None,
                None,
                Some("invalid-address".to_string()),
            )
            .await
            .unwrap_err();
            assert_eq!(err.to_string(), "Odd number of digits");
            assert_eq!(
                err.to_readable_msg(),
                "Invalid address format: Odd number of digits"
            );

            let err = do_quote_specs(
                get_batch_quote_specs(),
                "some-url".to_string(),
                "some-url".to_string(),
                None,
                Some("invalid-gas".to_string()),
                None,
            )
            .await
            .unwrap_err();
            assert_eq!(err.to_string(), "digit 18 is out of range for base 10");
            assert_eq!(
                err.to_readable_msg(),
                "Invalid numeric value: digit 18 is out of range for base 10"
            );
        }

        #[tokio::test]
        async fn test_get_batch_quote_target_from_subgraph() {
            let subgraph_server = MockServer::start_async().await;

            subgraph_server.mock(|when, then| {
                when.path("/subgraph");
                then.json_body_obj(&get_order_json());
            });

            let res = get_batch_quote_target_from_subgraph(
                get_batch_quote_specs(),
                subgraph_server.url("/subgraph"),
            )
            .await
            .unwrap();

            assert_eq!(res.0.len(), 2);
            match &res.0[0] {
                Some(QuoteTarget {
                    orderbook,
                    quote_config,
                }) => {
                    assert_eq!(
                        orderbook,
                        &Address::from_str("0x1000000000000000000000000000000000000000").unwrap()
                    );
                    assert_eq!(quote_config, &get_quote_config());
                }
                None => panic!("Expected quote target, got none"),
            }
            match &res.0[1] {
                Some(QuoteTarget {
                    orderbook,
                    quote_config,
                }) => {
                    assert_eq!(
                        orderbook,
                        &Address::from_str("0x2000000000000000000000000000000000000000").unwrap()
                    );
                    assert_eq!(
                        quote_config,
                        &Quote {
                            inputIOIndex: U256::from(1),
                            outputIOIndex: U256::from(1),
                            ..get_quote_config()
                        }
                    );
                }
                None => panic!("Expected quote target, got none"),
            }
        }

        #[tokio::test]
        async fn test_get_order_quote() {
            let rpc_server = MockServer::start_async().await;

            let order = SgOrder {
                id: SgBytes("0x46891c626a8a188610b902ee4a0ce8a7e81915e1b922584f8168d14525899dfb".to_string()),
                order_bytes:
                    SgBytes("0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001111111111111111111111111111111111111111000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000222222222222222222222222222222222222222200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string()),
                order_hash: SgBytes("0x283508c8f56f4de2f21ee91749d64ec3948c16bc6b4bfe4f8d11e4e67d76f4e0".to_string()),
                owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                outputs: vec![SgVault {
                    id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        address: SgBytes("0x2222222222222222222222222222222222222222".to_string()),
                        name: Some("T1".to_string()),
                        symbol: Some("T1".to_string()),
                        decimals: Some(SgBigInt("0".to_string())),
                    },
                    balance: SgBigInt("0".to_string()),
                    vault_id: SgBigInt("0".to_string()),
                    owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    orders_as_output: vec![],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                    orderbook: SgOrderbook {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    },
                }],
                inputs: vec![SgVault {
                    id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        address: SgBytes("0x1111111111111111111111111111111111111111".to_string()),
                        name: Some("T2".to_string()),
                        symbol: Some("T2".to_string()),
                        decimals: Some(SgBigInt("0".to_string())),
                    },
                    balance: SgBigInt("0".to_string()),
                    vault_id: SgBigInt("0".to_string()),
                    owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    orders_as_output: vec![],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                    orderbook: SgOrderbook {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    },
                }],
                active: true,
                add_events: vec![
                    SgAddOrder {
                        transaction: SgTransaction {
                            id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                            block_number: SgBigInt("0".to_string()),
                            timestamp: SgBigInt("0".to_string()),
                            from: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        },
                    }
                ],
                meta: None,
                timestamp_added: SgBigInt("0".to_string()),
                orderbook: SgOrderbook {
                    id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                },
                trades: vec![],
                remove_events: vec![]
            };

            // block number 1
            rpc_server.mock(|when, then| {
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
            rpc_server.mock(|when, then| {
                when.path("/rpc");
                then.body(
                    Response::new_success(1, &response_hex)
                        .to_json_string()
                        .unwrap(),
                );
            });

            let res = get_order_quote(vec![order], rpc_server.url("/rpc"), None, None)
                .await
                .unwrap();
            assert_eq!(res.0.len(), 1);
            assert_eq!(res.0[0].data.unwrap().max_output, U256::from(1));
            assert_eq!(res.0[0].data.unwrap().ratio, U256::from(2));
            assert_eq!(res.0[0].success, true);
            assert_eq!(res.0[0].error, None);
            assert_eq!(res.0[0].pair.pair_name, "T2/T1");
            assert_eq!(res.0[0].pair.input_index, 0);
            assert_eq!(res.0[0].pair.output_index, 0);
        }

        #[tokio::test]
        async fn test_get_order_quote_invalid_values() {
            let err = get_order_quote(
                vec![],
                "some-url".to_string(),
                None,
                Some("invalid-gas".to_string()),
            )
            .await
            .unwrap_err();
            assert_eq!(err.to_string(), "digit 18 is out of range for base 10");
            assert_eq!(
                err.to_readable_msg(),
                "Invalid numeric value: digit 18 is out of range for base 10"
            );
        }
    }
}
