use crate::{
    error::{Error, FailedQuote},
    rpc::batch_quote,
};
use alloy_primitives::{
    hex::{decode, encode_prefixed},
    Address, U256,
};
use alloy_sol_types::SolValue;
use rain_orderbook_bindings::IOrderBookV4::{quoteReturn, OrderV3, Quote, SignedContextV1};
use rain_orderbook_subgraph_client::{
    types::{order_detail::Bytes, Id},
    OrderbookSubgraphClient,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

pub type QuoteResult = Result<OrderQuote, FailedQuote>;

/// Holds quoted order max output and ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderQuote {
    pub max_output: U256,
    pub ratio: U256,
}

impl From<quoteReturn> for OrderQuote {
    fn from(v: quoteReturn) -> Self {
        Self {
            max_output: v.outputMax,
            ratio: v.ioRatio,
        }
    }
}

/// A quote target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteTarget {
    pub order_id: U256,
    pub quote_config: Quote,
    pub orderbook: Address,
}

/// A quote target specifier, where the order details need to be fetched from a
/// source (such as subgraph) to build a [QuoteTarget] out of it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteTargetSpecifier {
    pub order_id: U256,
    pub input_io_index: U256,
    pub output_io_index: U256,
    pub signed_context: Vec<SignedContextV1>,
    pub orderbook: Address,
}

impl QuoteTargetSpecifier {
    /// Given a quote specifier will fetch its order details and returns the
    /// respective quote target
    pub async fn get_quote_target_from_subgraph(
        &self,
        subgraph_url: &str,
    ) -> Result<QuoteTarget, Error> {
        let url = Url::from_str(subgraph_url)?;
        let sg_client = OrderbookSubgraphClient::new(url);
        let order_detail = sg_client
            .order_detail(Id::new(encode_prefixed(self.order_id.to_be_bytes_vec())))
            .await?;

        Ok(QuoteTarget {
            order_id: self.order_id,
            orderbook: self.orderbook,
            quote_config: Quote {
                inputIOIndex: self.input_io_index,
                outputIOIndex: self.output_io_index,
                signedContext: self.signed_context.clone(),
                order: OrderV3::abi_decode(
                    decode(order_detail.order_bytes.0.as_str())?.as_slice(),
                    true,
                )?,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchQuoteTargetSpecifier(pub Vec<QuoteTargetSpecifier>);

impl BatchQuoteTargetSpecifier {
    /// Given a list of quote specifiers, will fetch their order details and
    /// returns their respective quote targets.
    /// It will only return quote targets
    /// fro those quote specifiers that were actually available on the given subgraph
    pub async fn get_batch_quote_target_from_subgraph(
        &self,
        subgraph_url: &str,
    ) -> Result<Vec<QuoteTarget>, Error> {
        let url = Url::from_str(subgraph_url)?;
        let sg_client = OrderbookSubgraphClient::new(url);
        let orders_details = sg_client
            .batch_order_detail(
                self.0
                    .iter()
                    .map(|v| Bytes(encode_prefixed(v.order_id.to_be_bytes_vec())))
                    .collect(),
            )
            .await?;

        Ok(self
            .0
            .iter()
            .filter_map(|v| {
                orders_details
                    .iter()
                    .find(|e| e.id.0 == encode_prefixed(v.order_id.to_be_bytes_vec()))
                    .and_then(|order_detail| {
                        Some(QuoteTarget {
                            order_id: v.order_id,
                            orderbook: v.orderbook,
                            quote_config: Quote {
                                inputIOIndex: v.input_io_index,
                                outputIOIndex: v.output_io_index,
                                signedContext: v.signed_context.clone(),
                                order: OrderV3::abi_decode(
                                    decode(order_detail.order_bytes.0.as_str()).ok()?.as_slice(),
                                    true,
                                )
                                .ok()?,
                            },
                        })
                    })
            })
            .collect())
    }
}

/// The main struct providing functionalities to easily quote orderbook orders
#[derive(Debug, Clone)]
pub struct Quoter;

impl Quoter {
    /// Given a list of quote specifiers and a subgraph url, will fetch the
    /// order details from the subgraph and then quotes them using the given
    /// rpc url
    pub async fn quote_from_subgraph(
        subgraph_url: &str,
        batch_quote_target_specifier: &BatchQuoteTargetSpecifier,
        rpc_url: &str,
        block_number: Option<u64>,
        multicall_address: Option<Address>,
    ) -> Result<Vec<QuoteResult>, Error> {
        let quote_targets = batch_quote_target_specifier
            .get_batch_quote_target_from_subgraph(subgraph_url)
            .await?;

        batch_quote(&quote_targets, rpc_url, block_number, multicall_address).await
    }

    /// Quotes the given targets on the given rpc url
    pub async fn quote(
        quote_targets: &[QuoteTarget],
        rpc_url: &str,
        block_number: Option<u64>,
        multicall_address: Option<Address>,
    ) -> Result<Vec<QuoteResult>, Error> {
        batch_quote(quote_targets, rpc_url, block_number, multicall_address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_ethers_typecast::multicall::IMulticall3::Result as MulticallResult;
    use alloy_ethers_typecast::multicall::IMulticall3::{aggregate3Call, Call3};
    use alloy_ethers_typecast::multicall::MULTICALL3_ADDRESS;
    use alloy_ethers_typecast::request_shim::{AlloyTransactionRequest, TransactionRequestShim};
    use alloy_ethers_typecast::rpc::eip2718::TypedTransaction;
    use alloy_ethers_typecast::rpc::{BlockNumber, Request, Response};
    use alloy_primitives::hex::FromHex;
    use alloy_primitives::keccak256;
    use alloy_primitives::{hex::encode_prefixed, U256};
    use alloy_sol_types::{SolCall, SolValue};
    use httpmock::{Method::POST, MockServer};
    use rain_orderbook_bindings::IOrderBookV4::{quoteCall, Quote, IO};
    use serde_json::{from_str, json, Value};

    fn get_order_helper(batch: bool) -> (Address, OrderV3, U256, Value) {
        let orderbook = Address::random();
        let order = OrderV3 {
            validInputs: vec![IO {
                ..Default::default()
            }],
            validOutputs: vec![IO {
                ..Default::default()
            }],
            ..Default::default()
        };
        let order_id_u256 = U256::from_be_bytes(keccak256(encode_prefixed(order.abi_encode())).0);
        let order_id = encode_prefixed(keccak256(encode_prefixed(order.abi_encode())));
        let order_json = json!({
            "id": order_id,
            "orderBytes": encode_prefixed(order.abi_encode()),
            "orderHash": order_id,
            "owner": encode_prefixed(order.owner),
            "outputs": [{
                "id": encode_prefixed(Address::random().0.0),
                "token": {
                    "id": encode_prefixed(order.validOutputs[0].token.0.0),
                    "address": encode_prefixed(order.validOutputs[0].token.0.0),
                    "name": "T1",
                    "symbol": "T1",
                    "decimals": order.validOutputs[0].decimals.to_string()
                },
                "balance": "0",
                "vaultId": order.validOutputs[0].vaultId.to_string(),
            }],
            "inputs": [{
                "id": encode_prefixed(Address::random().0.0),
                "token": {
                    "id": encode_prefixed(order.validInputs[0].token.0.0),
                    "address": encode_prefixed(order.validInputs[0].token.0.0),
                    "name": "T2",
                    "symbol": "T2",
                    "decimals": order.validInputs[0].decimals.to_string()
                },
                "balance": "0",
                "vaultId": order.validInputs[0].vaultId.to_string(),
            }],
            "active": true,
            "addEvents": [{
                "transaction": {
                    "blockNumber": "0",
                    "timestamp": "0"
                }
            }],
            "meta": null,
            "timestampAdded": "0",
        });
        let retrun_sg_data = if batch {
            json!({
                "data": {
                    "orders": [order_json]
                }
            })
        } else {
            json!({
                "data": {
                    "order": order_json
                }
            })
        };
        (orderbook, order, order_id_u256, retrun_sg_data)
    }

    #[tokio::test]
    async fn test_get_quote_target_from_subgraph() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, order_id_u256, retrun_sg_data) = get_order_helper(false);

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body_obj(&retrun_sg_data);
        });

        let quote_target_specifier = QuoteTargetSpecifier {
            order_id: order_id_u256,
            input_io_index: U256::ZERO,
            output_io_index: U256::ZERO,
            signed_context: vec![],
            orderbook,
        };
        let result = quote_target_specifier
            .get_quote_target_from_subgraph(rpc_server.url("/").as_str())
            .await
            .unwrap();

        let expected = QuoteTarget {
            order_id: order_id_u256,
            orderbook,
            quote_config: Quote {
                order,
                inputIOIndex: quote_target_specifier.input_io_index,
                outputIOIndex: quote_target_specifier.output_io_index,
                signedContext: quote_target_specifier.signed_context,
            },
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_batch_quote_target_from_subgraph() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, order_id_u256, retrun_sg_data) = get_order_helper(true);

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body_obj(&retrun_sg_data);
        });

        let batch_quote_targets_specifiers =
            BatchQuoteTargetSpecifier(vec![QuoteTargetSpecifier {
                order_id: order_id_u256,
                input_io_index: U256::ZERO,
                output_io_index: U256::ZERO,
                signed_context: vec![],
                orderbook,
            }]);
        let result = batch_quote_targets_specifiers
            .get_batch_quote_target_from_subgraph(rpc_server.url("/").as_str())
            .await
            .unwrap();

        let expected = vec![QuoteTarget {
            order_id: order_id_u256,
            orderbook,
            quote_config: Quote {
                order,
                inputIOIndex: batch_quote_targets_specifiers.0[0].input_io_index,
                outputIOIndex: batch_quote_targets_specifiers.0[0].output_io_index,
                signedContext: batch_quote_targets_specifiers.0[0].signed_context.clone(),
            },
        }];

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_quoter_quote_from_subgraph() {
        let rpc_server = MockServer::start_async().await;

        let multicall = Address::from_hex(MULTICALL3_ADDRESS).unwrap();
        let (orderbook, order, order_id_u256, retrun_sg_data) = get_order_helper(true);

        let quote_targets = vec![QuoteTarget {
            order_id: order_id_u256,
            quote_config: Quote {
                order: order.clone(),
                ..Default::default()
            },
            orderbook,
        }];
        let call = aggregate3Call {
            calls: quote_targets
                .iter()
                .map(|quote_target| Call3 {
                    allowFailure: true,
                    target: quote_target.orderbook,
                    callData: quoteCall {
                        quoteConfig: quote_target.quote_config.clone(),
                    }
                    .abi_encode(),
                })
                .collect(),
        };

        // build response data
        let response_data = vec![MulticallResult {
            success: true,
            returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2))),
        }]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc").json_body_partial(
                Request::<(TypedTransaction, BlockNumber)>::eth_call_request(
                    1,
                    TypedTransaction::Eip1559(
                        AlloyTransactionRequest::new()
                            .with_to(Some(multicall))
                            .with_data(Some(call.abi_encode()))
                            .to_eip1559(),
                    ),
                    None,
                )
                .to_json_string()
                .unwrap(),
            );
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/sg");
            then.json_body_obj(&retrun_sg_data);
        });

        let batch_quote_targets_specifiers =
            BatchQuoteTargetSpecifier(vec![QuoteTargetSpecifier {
                order_id: order_id_u256,
                input_io_index: U256::ZERO,
                output_io_index: U256::ZERO,
                signed_context: vec![],
                orderbook,
            }]);

        let result = Quoter::quote_from_subgraph(
            rpc_server.url("/sg").as_str(),
            &batch_quote_targets_specifiers,
            rpc_server.url("/rpc").as_str(),
            None,
            None,
        )
        .await
        .unwrap();
        let mut iter_result = result.into_iter();

        assert_eq!(
            iter_result.next().unwrap().unwrap(),
            OrderQuote {
                max_output: U256::from(1),
                ratio: U256::from(2),
            }
        );
        assert!(iter_result.next().is_none());
    }

    #[tokio::test]
    async fn test_quoter_quote() {
        let rpc_server = MockServer::start_async().await;

        // let orderbook = Address::random();
        let multicall = Address::from_hex(MULTICALL3_ADDRESS).unwrap();

        let (orderbook, order, order_id_u256, _) = get_order_helper(false);
        let quote_targets = vec![QuoteTarget {
            order_id: order_id_u256,
            quote_config: Quote {
                order: order.clone(),
                ..Default::default()
            },
            orderbook,
        }];
        let call = aggregate3Call {
            calls: quote_targets
                .iter()
                .map(|quote_target| Call3 {
                    allowFailure: true,
                    target: quote_target.orderbook,
                    callData: quoteCall {
                        quoteConfig: quote_target.quote_config.clone(),
                    }
                    .abi_encode(),
                })
                .collect(),
        };

        // build response data
        let response_data = vec![MulticallResult {
            success: true,
            returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2))),
        }]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc").json_body_partial(
                Request::<(TypedTransaction, BlockNumber)>::eth_call_request(
                    1,
                    TypedTransaction::Eip1559(
                        AlloyTransactionRequest::new()
                            .with_to(Some(multicall))
                            .with_data(Some(call.abi_encode()))
                            .to_eip1559(),
                    ),
                    None,
                )
                .to_json_string()
                .unwrap(),
            );
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        let result = Quoter::quote(&quote_targets, rpc_server.url("/rpc").as_str(), None, None)
            .await
            .unwrap();
        let mut iter_result = result.into_iter();

        assert_eq!(
            iter_result.next().unwrap().unwrap(),
            OrderQuote {
                max_output: U256::from(1),
                ratio: U256::from(2),
            }
        );
        assert!(iter_result.next().is_none());
    }
}
