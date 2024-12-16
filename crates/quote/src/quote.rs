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
    types::{common::Bytes, Id},
    utils::make_order_id,
    OrderbookSubgraphClient,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, str::FromStr};
use typeshare::typeshare;
use url::Url;

pub type QuoteResult = Result<OrderQuoteValue, FailedQuote>;

/// Holds quoted order max output and ratio
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteValue {
    #[typeshare(typescript(type = "string"))]
    pub max_output: U256,
    #[typeshare(typescript(type = "string"))]
    pub ratio: U256,
}

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
pub struct QuoteTarget {
    pub quote_config: Quote,
    pub orderbook: Address,
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(transparent)]
#[serde(rename_all = "camelCase")]
pub struct BatchQuoteTarget(pub Vec<QuoteTarget>);

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
pub struct QuoteSpec {
    pub order_hash: U256,
    pub input_io_index: u8,
    pub output_io_index: u8,
    pub signed_context: Vec<SignedContextV1>,
    pub orderbook: Address,
}

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
            .order_detail(Id::new(encode_prefixed(self.get_id())))
            .await?;

        Ok(QuoteTarget {
            orderbook: self.orderbook,
            quote_config: Quote {
                inputIOIndex: U256::from(self.input_io_index),
                outputIOIndex: U256::from(self.output_io_index),
                signedContext: self.signed_context.clone(),
                order: OrderV3::abi_decode(
                    decode(order_detail.order_bytes.0.as_str())?.as_slice(),
                    true,
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
        gas: Option<U256>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(transparent)]
#[serde(rename_all = "camelCase")]
pub struct BatchQuoteSpec(pub Vec<QuoteSpec>);

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
                    .map(|v| Bytes(encode_prefixed(v.get_id())))
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
    use alloy::primitives::keccak256;
    use alloy::primitives::{hex::encode_prefixed, U256};
    use alloy::sol_types::{SolCall, SolValue};
    use alloy_ethers_typecast::multicall::IMulticall3::Result as MulticallResult;
    use alloy_ethers_typecast::rpc::Response;
    use httpmock::{Method::POST, MockServer};
    use rain_orderbook_bindings::IOrderBookV4::{quoteCall, Quote, IO};
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
            "trades": []
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

    #[tokio::test]
    async fn test_get_quote_spec_from_subgraph() {
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
    async fn test_get_batch_quote_spec_from_subgraph() {
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
    async fn test_quote_spec_do_quote() {
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
    async fn test_quote_batch_spec_do_quote() {
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
            // should be None in final result
            QuoteSpec::default(),
            QuoteSpec::default(),
        ]);

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

        // specifiers that were not present on subgraph were not quoted and are None
        assert!(iter_result.next().unwrap().is_err());
        assert!(iter_result.next().unwrap().is_err());

        // all results should have been consumed by now
        assert!(iter_result.next().is_none());
    }

    #[tokio::test]
    async fn test_quote_target_do_quote() {
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
    async fn test_batch_quote_target_do_quote() {
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
}
