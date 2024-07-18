use crate::{
    error::{Error, FailedQuote},
    quote::{QuoteResult, QuoteTarget},
    BatchQuoteTargetSpecifier, Quoter,
};
use alloy_ethers_typecast::multicall::{
    IMulticall3::{aggregate3Call, Call3},
    MULTICALL3_ADDRESS,
};
use alloy_primitives::{hex::FromHex, Address};
use alloy_sol_types::SolCall;
use once_cell::sync::Lazy;
use rain_error_decoding::AbiDecodedErrorType;
use rain_interpreter_eval::fork::{Forker, NewForkedEvm};
use rain_orderbook_bindings::IOrderBookV4::quoteCall;
use std::sync::Arc;
use tokio::sync::Mutex;

pub static FORKER: Lazy<Arc<Mutex<Forker>>> = Lazy::new(|| Arc::new(Mutex::new(Forker::new())));

/// Quotes array of given orders on a fork using multicall
pub async fn fork_multi_quote(
    quote_targets: &[QuoteTarget],
    rpc_url: &str,
    fork_block_number: Option<u64>,
    multicall_address: Option<Address>,
) -> Result<Vec<QuoteResult>, Error> {
    let mut forker = FORKER.lock().await;
    forker
        .add_or_select(
            NewForkedEvm {
                fork_url: rpc_url.to_string(),
                fork_block_number,
            },
            None,
        )
        .await?;

    let call_result = forker
        .alloy_call(
            Address::random(),
            multicall_address.unwrap_or(Address::from_hex(MULTICALL3_ADDRESS).unwrap()),
            aggregate3Call {
                calls: quote_targets
                    .iter()
                    .map(|quote_target| Call3 {
                        allowFailure: true,
                        target: quote_target.orderbook,
                        callData: quoteCall {
                            quoteConfig: quote_target.quote.clone(),
                        }
                        .abi_encode(),
                    })
                    .collect(),
            },
            true,
        )
        .await?;

    let mut result: Vec<QuoteResult> = vec![];
    for res in call_result.typed_return.returnData {
        if res.success {
            match quoteCall::abi_decode_returns(&res.returnData, true) {
                Ok(v) => {
                    if v.exists {
                        result.push(Ok(v.into()));
                    } else {
                        result.push(Err(FailedQuote::NonExistent));
                    }
                }
                Err(e) => result.push(Err(FailedQuote::CorruptReturnData(e.to_string()))),
            }
        } else {
            match AbiDecodedErrorType::selector_registry_abi_decode(&res.returnData).await {
                Ok(e) => result.push(Err(FailedQuote::RevertError(e))),
                Err(e) => result.push(Err(FailedQuote::RevertErrorDecodeFailed(e))),
            }
        }
    }
    Ok(result)
}

// impl fork related methods for quoter
impl Quoter {
    /// Given a list of quote specifiers and a subgraph url, will fetch the
    /// order details from the subgraph and then quotes them on a fork with
    /// the given fork's details
    pub async fn fork_quote_from_subgraph(
        subgraph_url: &str,
        batch_quote_target_specifier: &BatchQuoteTargetSpecifier,
        rpc_url: &str,
        fork_block_number: Option<u64>,
        multicall_address: Option<Address>,
    ) -> Result<Vec<QuoteResult>, Error> {
        let quote_targets = batch_quote_target_specifier
            .get_batch_quote_target_from_subgraph(subgraph_url)
            .await?;

        fork_multi_quote(
            &quote_targets,
            rpc_url,
            fork_block_number,
            multicall_address,
        )
        .await
    }

    /// Quotes the given targets on a fork with the given fork's details
    pub async fn fork_quote(
        quote_targets: &[QuoteTarget],
        rpc_url: &str,
        fork_block_number: Option<u64>,
        multicall_address: Option<Address>,
    ) -> Result<Vec<QuoteResult>, Error> {
        fork_multi_quote(quote_targets, rpc_url, fork_block_number, multicall_address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{quote::OrderQuote, QuoteTargetSpecifier};
    use alloy_primitives::{hex::decode, hex::encode_prefixed, keccak256, U256};
    use alloy_sol_types::SolValue;
    use httpmock::{Method::POST, MockServer};
    use rain_orderbook_bindings::IOrderBookV4::{
        addOrder2Call, EvaluableV3, OrderConfigV3, OrderV3, Quote, IO,
    };
    use serde_json::json;

    // helper fn to deploy an order on the fork
    async fn deploy_order_helper(
        ratio: &str,
        max_output: &str,
        block_number: u64,
    ) -> (OrderV3, Address, String) {
        let rpc_url =
            std::env::var("CI_DEPLOY_FLARE_RPC_URL").expect("undefined flare network rpc");

        // known contracts
        let orderbook = Address::from_hex("0x582d9e838FE6cD9F8147C66A8f56A3FBE513a6A2").unwrap();
        let store = Address::from_hex("0xe106d53C422a7858a00516Ff22A4485544f56BD3").unwrap();
        let interpreter = Address::from_hex("0xf29487D3f4B8262714179546cF7419c0A7cC0BeC").unwrap();

        // build an order
        let owner = Address::random();
        let evaluable = EvaluableV3 {
            interpreter,
            store,
            // equates to rainlang:
            // _ _: max_output ratio; :;
            bytecode: decode(
                format!(
                    "0x0000000000000000000000000000000000000000000000000000000000000002{}{}0000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000100000000",
                    max_output,
                    ratio
                )
            ).unwrap(),
        };
        let valid_inputs = vec![IO {
            ..Default::default()
        }];
        let valid_outputs = vec![IO {
            ..Default::default()
        }];
        let order = OrderV3 {
            owner,
            evaluable: evaluable.clone(),
            validInputs: valid_inputs.clone(),
            validOutputs: valid_outputs.clone(),
            ..Default::default()
        };
        let order_config = OrderConfigV3 {
            evaluable,
            validInputs: valid_inputs,
            validOutputs: valid_outputs,
            ..Default::default()
        };
        let add_order = addOrder2Call {
            config: order_config,
            post: vec![],
        };

        // add the order to the fork
        let mut forker = FORKER.lock().await;
        forker
            .add_or_select(
                NewForkedEvm {
                    fork_url: rpc_url.to_string(),
                    fork_block_number: Some(block_number),
                },
                None,
            )
            .await
            .unwrap();

        forker
            .alloy_call_committing(owner, orderbook, add_order, U256::ZERO, true)
            .await
            .unwrap();

        (order, orderbook, rpc_url)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fork_multi_quote() {
        let ratio = "f".repeat(64);
        let max_output = "2".repeat(64);
        let block_number = 26938000;
        let (order, orderbook, rpc_url) =
            deploy_order_helper(&ratio, &max_output, block_number).await;

        let quote_targets = vec![
            QuoteTarget {
                id: U256::ZERO,
                quote: Quote {
                    order,
                    ..Default::default()
                },
                orderbook,
            },
            // not exists
            QuoteTarget {
                id: U256::ZERO,
                quote: Quote {
                    ..Default::default()
                },
                orderbook,
            },
        ];
        let result = fork_multi_quote(&quote_targets, &rpc_url, Some(block_number), None)
            .await
            .unwrap();
        let mut iter_result = result.into_iter();

        assert_eq!(
            iter_result.next().unwrap().unwrap(),
            OrderQuote {
                max_output: U256::ZERO,
                ratio: U256::MAX,
            }
        );
        matches!(
            iter_result.next().unwrap(),
            Result::Err(FailedQuote::NonExistent)
        );
        assert!(iter_result.next().is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_quoter_fork_quote_from_subgraph() {
        let rpc_server = MockServer::start_async().await;
        let ratio = "f".repeat(64);
        let max_output = "2".repeat(64);
        let block_number = 26938000;
        let (order, orderbook, rpc_url) =
            deploy_order_helper(&ratio, &max_output, block_number).await;

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

        let result = Quoter::fork_quote_from_subgraph(
            rpc_server.url("/").as_str(),
            &batch_quote_targets_specifiers,
            &rpc_url,
            Some(block_number),
            None,
        )
        .await
        .unwrap();
        let mut iter_result = result.into_iter();

        assert_eq!(
            iter_result.next().unwrap().unwrap(),
            OrderQuote {
                max_output: U256::ZERO,
                ratio: U256::MAX,
            }
        );
        assert!(iter_result.next().is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_quoter_fork_quote() {
        let ratio = "f".repeat(64);
        let max_output = "2".repeat(64);
        let block_number = 26938000;
        let (order, orderbook, rpc_url) =
            deploy_order_helper(&ratio, &max_output, block_number).await;

        let quote_targets = vec![QuoteTarget {
            id: U256::ZERO,
            quote: Quote {
                order,
                ..Default::default()
            },
            orderbook,
        }];

        let result = Quoter::fork_quote(&quote_targets, &rpc_url, Some(block_number), None)
            .await
            .unwrap();
        let mut iter_result = result.into_iter();

        assert_eq!(
            iter_result.next().unwrap().unwrap(),
            OrderQuote {
                max_output: U256::ZERO,
                ratio: U256::MAX,
            }
        );
        assert!(iter_result.next().is_none());
    }
}
