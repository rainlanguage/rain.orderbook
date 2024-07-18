use crate::error::{Error, FailedQuote};
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

/// The main struct providing functionalities to easily quote orderbook orders
#[derive(Debug, Clone)]
pub struct Quoter;

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
    pub id: U256,
    pub quote: Quote,
    pub orderbook: Address,
}

/// A quote target specifier, where the order details need to be fetched from a
/// source (such as subgraph) to build a [QuoteTarget] out of it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteTargetSpecifier {
    pub id: U256,
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
            .order_detail(Id::new(encode_prefixed(self.id.to_be_bytes_vec())))
            .await?;

        Ok(QuoteTarget {
            id: self.id,
            orderbook: self.orderbook,
            quote: Quote {
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
                    .map(|v| Bytes(encode_prefixed(v.id.to_be_bytes_vec())))
                    .collect(),
            )
            .await?;

        Ok(self
            .0
            .iter()
            .filter_map(|v| {
                orders_details
                    .iter()
                    .find(|e| e.id.0 == encode_prefixed(v.id.to_be_bytes_vec()))
                    .and_then(|order_detail| {
                        Some(QuoteTarget {
                            id: v.id,
                            orderbook: v.orderbook,
                            quote: Quote {
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::keccak256;
    use alloy_primitives::{hex::encode_prefixed, U256};
    use alloy_sol_types::SolValue;
    use httpmock::{Method::POST, MockServer};
    use rain_orderbook_bindings::IOrderBookV4::{Quote, IO};
    use serde_json::json;

    #[tokio::test]
    async fn test_get_quote_target_from_subgraph() {
        let rpc_server = MockServer::start_async().await;

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
        let retrun_sg_data = json!({
            "data": {
                "order": {
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
                }
            }
        });

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body_obj(&retrun_sg_data);
        });

        let quote_target_specifier = QuoteTargetSpecifier {
            id: order_id_u256,
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
            id: order_id_u256,
            orderbook,
            quote: Quote {
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
        let retrun_sg_data = json!({
            "data": {
                "orders": [{
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
                }]
            }
        });

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body_obj(&retrun_sg_data);
        });

        let batch_quote_targets_specifiers =
            BatchQuoteTargetSpecifier(vec![QuoteTargetSpecifier {
                id: order_id_u256,
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
            id: order_id_u256,
            orderbook,
            quote: Quote {
                order,
                inputIOIndex: batch_quote_targets_specifiers.0[0].input_io_index,
                outputIOIndex: batch_quote_targets_specifiers.0[0].output_io_index,
                signedContext: batch_quote_targets_specifiers.0[0].signed_context.clone(),
            },
        }];

        assert_eq!(result, expected);
    }
}
