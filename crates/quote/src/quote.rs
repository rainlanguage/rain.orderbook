use crate::{
    error::{Error, FailedQuote},
    rpc::batch_quote,
};
use alloy::primitives::{
    hex::{decode, encode_prefixed},
    keccak256, Address, B256, U256,
};
use alloy::sol_types::SolValue;
use rain_orderbook_bindings::IOrderBookV4::{quoteReturn, OrderV3, Quote, SignedContextV1};
use rain_orderbook_subgraph_client::{
    types::{common::SgBytes, Id},
    utils::make_order_id,
    OrderbookSubgraphClient,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, str::FromStr};
use url::Url;
use wasm_bindgen_utils::{add_ts_content, impl_wasm_traits, prelude::*};

pub type QuoteResult = Result<OrderQuoteValue, FailedQuote>;
add_ts_content!("export type QuoteResult = OrderQuoteValue | string");

/// Holds quoted order max output and ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct OrderQuoteValue {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub max_output: U256,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub ratio: U256,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderQuoteValue);

impl From<quoteReturn> for OrderQuoteValue {
    fn from(v: quoteReturn) -> Self {
        Self {
            max_output: v.outputMax,
            ratio: v.ioRatio,
        }
    }
}

/// A quote target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct QuoteTarget {
    pub quote_config: Quote,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub orderbook: Address,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(QuoteTarget);

impl QuoteTarget {
    /// Get the order hash of self
    pub fn get_order_hash(&self) -> B256 {
        keccak256(self.quote_config.order.abi_encode())
    }

    /// Get subgraph represented "order_id" of self
    /// which is keccak256 of orderbook address concated with order hash
    pub fn get_id(&self) -> B256 {
        make_order_id(self.orderbook, self.get_order_hash().into())
    }

    /// Quotes the target on the given rpc url
    pub async fn do_quote(
        &self,
        rpc_url: &str,
        block_number: Option<u64>,
        gas: Option<U256>,
        multicall_address: Option<Address>,
    ) -> Result<QuoteResult, Error> {
        Ok(batch_quote(
            &[self.clone()],
            rpc_url,
            block_number,
            gas,
            multicall_address,
        )
        .await?
        .into_iter()
        .next()
        .unwrap())
    }

    /// Validate the quote target
    /// Checks if the requested input and output indexes are valid
    pub fn validate(&self) -> Result<(), Error> {
        if self.quote_config.inputIOIndex >= U256::from(self.quote_config.order.validInputs.len()) {
            return Err(Error::InvalidQuoteTarget(self.quote_config.inputIOIndex));
        }
        if self.quote_config.outputIOIndex >= U256::from(self.quote_config.order.validOutputs.len())
        {
            return Err(Error::InvalidQuoteTarget(self.quote_config.outputIOIndex));
        }
        Ok(())
    }
}

/// Specifies a batch of [QuoteTarget]s
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(transparent)]
#[serde(rename_all = "camelCase")]
pub struct BatchQuoteTarget(pub Vec<QuoteTarget>);
impl_wasm_traits!(BatchQuoteTarget);

impl BatchQuoteTarget {
    /// Quotes the targets in batch on the given rpc url
    pub async fn do_quote(
        &self,
        rpc_url: &str,
        block_number: Option<u64>,
        gas: Option<U256>,
        multicall_address: Option<Address>,
    ) -> Result<Vec<QuoteResult>, Error> {
        batch_quote(&self.0, rpc_url, block_number, gas, multicall_address).await
    }
}

/// A quote target specifier, where the order details need to be fetched from a
/// source (such as subgraph) to build a [QuoteTarget] out of it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct QuoteSpec {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub order_hash: U256,
    #[serde(rename = "inputIOIndex")]
    pub input_io_index: u8,
    #[serde(rename = "outputIOIndex")]
    pub output_io_index: u8,
    pub signed_context: Vec<SignedContextV1>,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub orderbook: Address,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(QuoteSpec);

impl QuoteSpec {
    /// Get subgraph represented "order_id" of self
    /// which is keccak256 of orderbook address concated with order hash
    pub fn get_id(&self) -> B256 {
        make_order_id(self.orderbook, self.order_hash)
    }

    /// Given a subgraph will fetch the order details and returns the
    /// respective quote target
    pub async fn get_quote_target_from_subgraph(
        &self,
        subgraph_url: &str,
    ) -> Result<QuoteTarget, Error> {
        let url = Url::from_str(subgraph_url)?;
        let sg_client = OrderbookSubgraphClient::new(url);
        let order_detail = sg_client
            .order_detail(&Id::new(encode_prefixed(self.get_id())))
            .await?;

        Ok(QuoteTarget {
            orderbook: self.orderbook,
            quote_config: Quote {
                inputIOIndex: U256::from(self.input_io_index),
                outputIOIndex: U256::from(self.output_io_index),
                signedContext: self.signed_context.clone(),
                order: OrderV3::abi_decode(
                    decode(order_detail.order_bytes.0.as_str())?.as_slice(),
                )?,
            },
        })
    }

    /// Given a subgraph url, will fetch the order details from the subgraph and
    /// then quotes it using the given rpc url.
    pub async fn do_quote(
        &self,
        subgraph_url: &str,
        rpc_url: &str,
        block_number: Option<u64>,
        gas: Option<u64>,
        multicall_address: Option<Address>,
    ) -> Result<QuoteResult, Error> {
        let quote_target = self.get_quote_target_from_subgraph(subgraph_url).await?;
        let quote_result = batch_quote(
            &[quote_target],
            rpc_url,
            block_number,
            gas,
            multicall_address,
        )
        .await?;

        Ok(quote_result.into_iter().next().unwrap())
    }
}

/// specifies a batch of [QuoteSpec]s
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(transparent)]
#[serde(rename_all = "camelCase")]
pub struct BatchQuoteSpec(pub Vec<QuoteSpec>);
impl_wasm_traits!(BatchQuoteSpec);

impl BatchQuoteSpec {
    /// Given a subgraph url, will fetch orders details and returns their
    /// respective quote targets.
    /// Those specifiers that were not in the subgraph are returned as None
    /// in the resturning array
    pub async fn get_batch_quote_target_from_subgraph(
        &self,
        subgraph_url: &str,
    ) -> Result<Vec<Option<QuoteTarget>>, Error> {
        let url = Url::from_str(subgraph_url)?;
        let sg_client = OrderbookSubgraphClient::new(url);
        let orders_details = sg_client
            .batch_order_detail(
                self.0
                    .iter()
                    .map(|v| SgBytes(encode_prefixed(v.get_id())))
                    .collect(),
            )
            .await?;

        Ok(self
            .0
            .iter()
            .map(|target| {
                orders_details
                    .iter()
                    .find(|order_detail| order_detail.id.0 == encode_prefixed(target.get_id()))
                    .and_then(|order_detail| {
                        Some(QuoteTarget {
                            orderbook: target.orderbook,
                            quote_config: Quote {
                                inputIOIndex: U256::from(target.input_io_index),
                                outputIOIndex: U256::from(target.output_io_index),
                                signedContext: target.signed_context.clone(),
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

    /// Given a subgraph url, will fetch the order details from the subgraph and
    /// then quotes them using the given rpc url.
    /// Those orders that are not found from subgraph are excluded from quoting,
    /// and final result also leaves their place in the array as None
    pub async fn do_quote(
        &self,
        subgraph_url: &str,
        rpc_url: &str,
        block_number: Option<u64>,
        gas: Option<U256>,
        multicall_address: Option<Address>,
    ) -> Result<Vec<QuoteResult>, Error> {
        let opts_quote_targets = self
            .get_batch_quote_target_from_subgraph(subgraph_url)
            .await?;

        // quote the valid quote targets
        let quote_targets: Vec<QuoteTarget> = opts_quote_targets
            .iter()
            .filter_map(|v| v.clone())
            .collect();
        let mut quote_results = VecDeque::from(
            batch_quote(
                &quote_targets,
                rpc_url,
                block_number,
                gas,
                multicall_address,
            )
            .await?,
        );

        // fill the array with quote results and invalid quote targets following
        // their original order
        let mut result = vec![];
        opts_quote_targets.iter().for_each(|v| {
            if v.is_some() {
                result.push(
                    quote_results
                        .pop_front()
                        .unwrap_or(Err(FailedQuote::NonExistent)),
                );
            } else {
                result.push(Err(FailedQuote::NonExistent))
            }
        });

        Ok(result)
    }
}

#[cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy::hex;
    use alloy::hex::ToHex;
    use alloy::primitives::{address, keccak256};
    use alloy::primitives::{hex::encode_prefixed, U256};
    use alloy::sol_types::{SolCall, SolValue};
    use alloy_ethers_typecast::multicall::IMulticall3::Result as MulticallResult;
    use alloy_ethers_typecast::rpc::Response;
    use alloy_ethers_typecast::transaction::ReadableClientError;
    use httpmock::{Method::POST, MockServer};
    use rain_error_decoding::AbiDecodedErrorType;
    use rain_orderbook_bindings::IOrderBookV4::{quoteCall, Quote, IO};
    use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
    use serde_json::{from_str, json, Value};

    // helper fn to build some test data
    fn get_test_data(batch: bool) -> (Address, OrderV3, U256, Value) {
        let orderbook = Address::random();
        let order = OrderV3 {
            validInputs: vec![IO::default()],
            validOutputs: vec![IO::default()],
            ..Default::default()
        };
        let order_hash_bytes = keccak256(order.abi_encode()).0;
        let order_hash_u256 = U256::from_be_bytes(order_hash_bytes);
        let order_hash = encode_prefixed(order_hash_bytes);
        let mut id = vec![];
        id.extend_from_slice(orderbook.as_ref());
        id.extend_from_slice(&order_hash_bytes);
        let order_id = encode_prefixed(keccak256(id));
        let order_json = json!({
            "id": order_id,
            "orderBytes": encode_prefixed(order.abi_encode()),
            "orderHash": order_hash,
            "owner": encode_prefixed(order.owner),
            "outputs": [{
                "id": encode_prefixed(Address::random().0.0),
                "owner": encode_prefixed(order.owner),
                "token": {
                    "id": encode_prefixed(order.validOutputs[0].token.0.0),
                    "address": encode_prefixed(order.validOutputs[0].token.0.0),
                    "name": "T1",
                    "symbol": "T1",
                    "decimals": order.validOutputs[0].decimals.to_string()
                },
                "balance": "0",
                "vaultId": order.validOutputs[0].vaultId.to_string(),
                "orderbook": { "id": encode_prefixed(B256::random()) },
                "ordersAsOutput": [{
                    "id": encode_prefixed(B256::random()),
                    "orderHash": encode_prefixed(B256::random()),
                    "active": true
                }],
                "ordersAsInput": [{
                    "id": encode_prefixed(B256::random()),
                    "orderHash": encode_prefixed(B256::random()),
                    "active": true
                }],
                "balanceChanges": [{
                    "__typename": "Withdrawal",
                    "id": encode_prefixed(B256::random()),
                    "amount": "0",
                    "newVaultBalance": "0",
                    "oldVaultBalance": "0",
                    "vault": {
                        "id": encode_prefixed(B256::random()),
                        "vaultId": encode_prefixed(B256::random()),
                        "token": {
                            "id": encode_prefixed(order.validOutputs[0].token.0.0),
                            "address": encode_prefixed(order.validOutputs[0].token.0.0),
                            "name": "T1",
                            "symbol": "T1",
                            "decimals": order.validOutputs[0].decimals.to_string()
                        },
                    },
                    "timestamp": "0",
                    "transaction": {
                        "id": encode_prefixed(B256::random()),
                        "blockNumber": "0",
                        "timestamp": "0",
                        "from": encode_prefixed(Address::random())
                    },
                    "orderbook": { "id": encode_prefixed(B256::random()) }
                }],
            }],
            "inputs": [{
                "id": encode_prefixed(Address::random().0.0),
                "owner": encode_prefixed(order.owner),
                "token": {
                    "id": encode_prefixed(order.validInputs[0].token.0.0),
                    "address": encode_prefixed(order.validInputs[0].token.0.0),
                    "name": "T2",
                    "symbol": "T2",
                    "decimals": order.validInputs[0].decimals.to_string()
                },
                "balance": "0",
                "vaultId": order.validInputs[0].vaultId.to_string(),
                "orderbook": { "id": encode_prefixed(B256::random()) },
                "ordersAsOutput": [{
                    "id": encode_prefixed(B256::random()),
                    "orderHash": encode_prefixed(B256::random()),
                    "active": true
                }],
                "ordersAsInput": [{
                    "id": encode_prefixed(B256::random()),
                    "orderHash": encode_prefixed(B256::random()),
                    "active": true
                }],
                "balanceChanges": [{
                    "__typename": "Withdrawal",
                    "id": encode_prefixed(B256::random()),
                    "amount": "0",
                    "newVaultBalance": "0",
                    "oldVaultBalance": "0",
                    "vault": {
                        "id": encode_prefixed(B256::random()),
                        "vaultId": encode_prefixed(B256::random()),
                        "token": {
                            "id": encode_prefixed(order.validOutputs[0].token.0.0),
                            "address": encode_prefixed(order.validOutputs[0].token.0.0),
                            "name": "T1",
                            "symbol": "T1",
                            "decimals": order.validOutputs[0].decimals.to_string()
                        },
                    },
                    "timestamp": "0",
                    "transaction": {
                        "id": encode_prefixed(B256::random()),
                        "blockNumber": "0",
                        "timestamp": "0",
                        "from": encode_prefixed(Address::random())
                    },
                    "orderbook": { "id": encode_prefixed(B256::random()) }
                }],
            }],
            "orderbook": { "id": encode_prefixed(B256::random()) },
            "active": true,
            "addEvents": [{
                "transaction": {
                    "id": encode_prefixed(B256::random()),
                    "blockNumber": "0",
                    "timestamp": "0",
                    "from": encode_prefixed(Address::random())
                }
            }],
            "meta": null,
            "timestampAdded": "0",
            "trades": [],
            "removeEvents": []
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
        (orderbook, order, order_hash_u256, retrun_sg_data)
    }

    #[test]
    fn test_quote_target_get_order_hash() {
        let (orderbook, order, _, _) = get_test_data(false);
        let quote_target = QuoteTarget {
            quote_config: Quote {
                order,
                ..Default::default()
            },
            orderbook,
        };
        let actual = quote_target.get_order_hash().encode_hex::<String>();
        let expected =
            "8a3fbb9caf53f18f1f78d90c48dbe4612bcd93285ed0fc033009b4a96ea2aaed".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_quote_target_get_id() {
        let quote_target = QuoteTarget {
            quote_config: Default::default(),
            orderbook: Address::ZERO,
        };
        let actual = quote_target.get_id().encode_hex::<String>();
        let expected =
            "3c220b0ff68b48f69ef802b5d39e6942218a1b843a1845ade53d1e2412135b63".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_quote_spec_get_id() {
        let quote_spec = QuoteSpec {
            order_hash: U256::from(0_u16),
            input_io_index: 0,
            output_io_index: 0,
            signed_context: Vec::new(),
            orderbook: Address::ZERO,
        };
        let actual = quote_spec.get_id().encode_hex::<String>();
        let expected =
            "a86d54e9aab41ae5e520ff0062ff1b4cbd0b2192bb01080a058bb170d84e6457".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_validate_ok() {
        let (orderbook, order, _, _) = get_test_data(false);
        let quote_target = QuoteTarget {
            quote_config: Quote {
                order,
                ..Default::default()
            },
            orderbook,
        };
        assert!(quote_target.validate().is_ok());
    }

    #[test]
    fn test_validate_err() {
        let quote_target = QuoteTarget {
            quote_config: Quote {
                order: OrderV3::default(),
                outputIOIndex: U256::from(1_u16),
                ..Default::default()
            },
            orderbook: Address::ZERO,
        };
        assert!(quote_target.validate().is_err());

        let quote_target = QuoteTarget {
            quote_config: Quote {
                order: OrderV3::default(),
                inputIOIndex: U256::from(1_u16),
                ..Default::default()
            },
            orderbook: Address::ZERO,
        };
        assert!(quote_target.validate().is_err());
    }

    #[tokio::test]
    async fn test_get_quote_spec_from_subgraph_ok() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, order_id_u256, retrun_sg_data) = get_test_data(false);

        // mock subgraph
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body_obj(&retrun_sg_data);
        });

        let quote_target_specifier = QuoteSpec {
            order_hash: order_id_u256,
            input_io_index: 0,
            output_io_index: 0,
            signed_context: vec![],
            orderbook,
        };
        let result = quote_target_specifier
            .get_quote_target_from_subgraph(rpc_server.url("/").as_str())
            .await
            .unwrap();

        let expected = QuoteTarget {
            orderbook,
            quote_config: Quote {
                order,
                inputIOIndex: U256::from(quote_target_specifier.input_io_index),
                outputIOIndex: U256::from(quote_target_specifier.output_io_index),
                signedContext: quote_target_specifier.signed_context,
            },
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_quote_spec_from_subgraph_err() {
        let (orderbook, _, order_id_u256, _) = get_test_data(false);

        let quote_target_specifier = QuoteSpec {
            order_hash: order_id_u256,
            input_io_index: 0,
            output_io_index: 0,
            signed_context: vec![],
            orderbook,
        };

        let err = quote_target_specifier
            .get_quote_target_from_subgraph("this will break")
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::UrlParseError(url::ParseError::RelativeUrlWithoutBase)
        ));

        let rpc_server = MockServer::start_async().await;

        rpc_server.mock(|when, then| {
            when.method(POST).path("/404");
            then.status(404);
        });

        let err = quote_target_specifier
            .get_quote_target_from_subgraph(rpc_server.url("/404").as_str())
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::SubgraphClientError(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_get_batch_quote_spec_from_subgraph_ok() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, order_id_u256, retrun_sg_data) = get_test_data(true);

        // mock subgraph
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body_obj(&retrun_sg_data);
        });

        let batch_quote_targets_specifiers = BatchQuoteSpec(vec![QuoteSpec {
            order_hash: order_id_u256,
            input_io_index: 0,
            output_io_index: 0,
            signed_context: vec![],
            orderbook,
        }]);
        let result = batch_quote_targets_specifiers
            .get_batch_quote_target_from_subgraph(rpc_server.url("/").as_str())
            .await
            .unwrap();

        let expected = vec![Some(QuoteTarget {
            orderbook,
            quote_config: Quote {
                order,
                inputIOIndex: U256::from(batch_quote_targets_specifiers.0[0].input_io_index),
                outputIOIndex: U256::from(batch_quote_targets_specifiers.0[0].output_io_index),
                signedContext: batch_quote_targets_specifiers.0[0].signed_context.clone(),
            },
        })];

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_batch_quote_spec_from_subgraph_err() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, order_id_u256, _) = get_test_data(true);

        rpc_server.mock(|when, then| {
            when.method(POST).path("/sg");

            let invalid_order_json = json!({
                "id": encode_prefixed(B256::random()),
                "orderBytes": encode_prefixed(order.abi_encode()),
                "orderHash": encode_prefixed(B256::random()),
                "owner": encode_prefixed(order.owner),
                "orderbook": { "id": encode_prefixed(B256::random()) },
                "active": true,
                "addEvents": [],
                "meta": null,
                "timestampAdded": "0",
                "trades": [],
                "removeEvents": []
            });

            then.json_body_obj(&json!({
                "data": {
                    "orders": [invalid_order_json]
                }
            }));
        });

        let batch_quote_targets_specifiers = BatchQuoteSpec(vec![QuoteSpec {
            order_hash: order_id_u256,
            input_io_index: 0,
            output_io_index: 0,
            signed_context: vec![],
            orderbook,
        }]);

        let err = batch_quote_targets_specifiers
            .get_batch_quote_target_from_subgraph(rpc_server.url("/sg").as_str())
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::SubgraphClientError(OrderbookSubgraphClientError::CynicClientError(cynic_err))
            if cynic_err.to_string().contains("error decoding response body")
        ));
    }

    #[tokio::test]
    async fn test_quote_spec_do_quote_ok() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, _, order_id_u256, retrun_sg_data) = get_test_data(false);

        // build response data
        let response_data = vec![MulticallResult {
            success: true,
            returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2))).into(),
        }]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        // mock subgraph
        rpc_server.mock(|when, then| {
            when.method(POST).path("/sg");
            then.json_body_obj(&retrun_sg_data);
        });

        let quote_target_specifier = QuoteSpec {
            order_hash: order_id_u256,
            input_io_index: 0,
            output_io_index: 0,
            signed_context: vec![],
            orderbook,
        };

        let result = quote_target_specifier
            .do_quote(
                rpc_server.url("/sg").as_str(),
                rpc_server.url("/rpc").as_str(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            result.unwrap(),
            OrderQuoteValue {
                max_output: U256::from(1),
                ratio: U256::from(2),
            }
        );
    }

    #[tokio::test]
    async fn test_quote_spec_do_quote_err() {
        let server = MockServer::start_async().await;

        let (orderbook, _, order_id_u256, retrun_sg_data) = get_test_data(false);

        let response_data = vec![MulticallResult {
            success: true,
            returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2))).into(),
        }]
        .abi_encode();

        server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        server.mock(|when, then| {
            when.method(POST).path("/bad-rpc");
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, "not good data")
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        server.mock(|when, then| {
            when.method(POST).path("/sg");
            then.json_body_obj(&retrun_sg_data);
        });

        server.mock(|when, then| {
            when.method(POST).path("/bad-sg");
            then.json_body_obj(&json!({ "data": null }));
        });

        let quote_target_specifier = QuoteSpec {
            order_hash: order_id_u256,
            input_io_index: 0,
            output_io_index: 0,
            signed_context: vec![],
            orderbook,
        };

        let err = quote_target_specifier
            .do_quote(
                server.url("/sg").as_str(),
                server.url("/bad-rpc").as_str(),
                None,
                None,
                None,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::RpcCallError(ReadableClientError::AbiDecodedErrorType(
                AbiDecodedErrorType::Unknown(data)
            )) if data.is_empty()
        ));

        let err = quote_target_specifier
            .do_quote(
                server.url("/bad-sg").as_str(),
                server.url("/rpc").as_str(),
                None,
                None,
                None,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::SubgraphClientError(OrderbookSubgraphClientError::CynicClientError(
                cynic_err,
            )) if cynic_err.to_string().contains("error decoding response body")
        ));
    }

    #[tokio::test]
    async fn test_quote_batch_spec_do_quote_err() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, _, order_id_u256, retrun_sg_data) = get_test_data(true);

        // build response data
        let response_data = vec![MulticallResult {
            success: true,
            returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2))).into(),
        }]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        // mock subgraph
        rpc_server.mock(|when, then| {
            when.method(POST).path("/sg");
            then.json_body_obj(&retrun_sg_data);
        });

        let batch_quote_targets_specifiers = BatchQuoteSpec(vec![
            QuoteSpec {
                order_hash: order_id_u256,
                input_io_index: 0,
                output_io_index: 0,
                signed_context: vec![],
                orderbook,
            },
            // should be Err in final result
            QuoteSpec::default(),
            QuoteSpec::default(),
        ]);

        let err = batch_quote_targets_specifiers
            .do_quote(
                rpc_server.url("/sg").as_str(),
                "bad rpc url",
                None,
                None,
                None,
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::RpcCallError(ReadableClientError::CreateReadableClientHttpError(url_err))
            if url_err.to_string().contains("relative URL without a base")
        ));

        let result = batch_quote_targets_specifiers
            .do_quote(
                rpc_server.url("/sg").as_str(),
                rpc_server.url("/rpc").as_str(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let mut iter_result = result.into_iter();
        assert_eq!(
            iter_result.next().unwrap().unwrap(),
            OrderQuoteValue {
                max_output: U256::from(1),
                ratio: U256::from(2),
            }
        );

        // specifiers that were not present on subgraph were not quoted and are Err
        assert!(iter_result.next().unwrap().is_err());
        assert!(iter_result.next().unwrap().is_err());

        // all results should have been consumed by now
        assert!(iter_result.next().is_none());
    }

    #[tokio::test]
    async fn test_quote_target_do_quote_ok() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, _, _) = get_test_data(false);
        let quote_target = QuoteTarget {
            quote_config: Quote {
                order,
                ..Default::default()
            },
            orderbook,
        };

        // build response data
        let response_data = vec![MulticallResult {
            success: true,
            returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2))).into(),
        }]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        let result = quote_target
            .do_quote(rpc_server.url("/rpc").as_str(), None, None, None)
            .await
            .unwrap();

        assert_eq!(
            result.unwrap(),
            OrderQuoteValue {
                max_output: U256::from(1),
                ratio: U256::from(2),
            }
        );
    }

    #[tokio::test]
    async fn test_quote_target_do_quote_err() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, _, _) = get_test_data(false);
        let quote_target = QuoteTarget {
            quote_config: Quote {
                order,
                ..Default::default()
            },
            orderbook,
        };

        let response_data = vec![MulticallResult {
            success: true,
            returnData: "corrupt data".into(),
        }]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        let err = quote_target
            .do_quote(rpc_server.url("/rpc").as_str(), None, None, None)
            .await
            .unwrap()
            .unwrap_err();

        assert!(matches!(
            err,
            FailedQuote::CorruptReturnData(msg)
            if msg == *"buffer overrun while deserializing"
        ));
    }

    #[tokio::test]
    async fn test_batch_quote_target_do_quote_ok() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, _, _) = get_test_data(true);
        let quote_targets = BatchQuoteTarget(vec![QuoteTarget {
            quote_config: Quote {
                order,
                ..Default::default()
            },
            orderbook,
        }]);

        // build response data
        let response_data = vec![MulticallResult {
            success: true,
            returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2))).into(),
        }]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        let result = quote_targets
            .do_quote(rpc_server.url("/rpc").as_str(), None, None, None)
            .await
            .unwrap();
        let mut iter_result = result.into_iter();

        assert_eq!(
            iter_result.next().unwrap().unwrap(),
            OrderQuoteValue {
                max_output: U256::from(1),
                ratio: U256::from(2),
            }
        );
        assert!(iter_result.next().is_none());
    }

    #[tokio::test]
    async fn test_batch_quote_target_do_quote_err() {
        let rpc_server = MockServer::start_async().await;

        let (orderbook, order, _, _) = get_test_data(true);
        let quote_targets = BatchQuoteTarget(vec![QuoteTarget {
            quote_config: Quote {
                order,
                ..Default::default()
            },
            orderbook,
        }]);

        rpc_server.mock(|when, then| {
            when.method(POST).path("/error-rpc");
            then.status(500).json_body("internal server error");
        });

        rpc_server.mock(|when, then| {
            when.method(POST).path("/reverted-rpc");

            let response_data = vec![MulticallResult {
                success: false,
                // 0x734bc71c is the selector for TokenSelfTrade
                returnData: hex!("734bc71c").to_vec().into(),
            }]
            .abi_encode();

            then.json_body_obj(
                &from_str::<Value>(
                    &Response::new_success(1, encode_prefixed(response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        let err = quote_targets
            .do_quote(
                rpc_server.url("/error-rpc").as_str(),
                Some(1),
                Some(U256::from(1000000)),
                Some(address!("aaaaaaaaaabbbbbbbbbbccccccccccdddddddddd")),
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::RpcCallError(ReadableClientError::AbiDecodedErrorType(
                AbiDecodedErrorType::Unknown(data)
            )) if data.is_empty()
        ));

        let results = quote_targets
            .do_quote(rpc_server.url("/reverted-rpc").as_str(), None, None, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        let err = results.into_iter().next().unwrap().unwrap_err();

        assert!(matches!(
            err,
            FailedQuote::RevertError(AbiDecodedErrorType::Known { name, .. })
            if name == "TokenSelfTrade"
        ));
    }
}
