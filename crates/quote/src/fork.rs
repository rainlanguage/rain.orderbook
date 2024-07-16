use crate::{error::FailedQuote, quote::QuoteResult};
use alloy_ethers_typecast::multicall::{
    IMulticall3::{aggregate3Call, Call3},
    MULTICALL3_ADDRESS,
};
use alloy_primitives::{hex::FromHex, Address};
use alloy_sol_types::SolCall;
use once_cell::sync::Lazy;
use rain_error_decoding::AbiDecodedErrorType;
use rain_interpreter_eval::error::ForkCallError;
use rain_interpreter_eval::fork::{Forker, NewForkedEvm};
use rain_orderbook_bindings::IOrderBookV4::{quoteCall, Quote};
use std::sync::Arc;
use tokio::sync::Mutex;

pub static FORKER: Lazy<Arc<Mutex<Forker>>> = Lazy::new(|| Arc::new(Mutex::new(Forker::new())));

/// quotes a single order on a fork
pub async fn fork_signle_quote(
    quote: &Quote,
    orderbook: Address,
    rpc_url: &str,
    fork_block_number: Option<u64>,
) -> QuoteResult {
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
            orderbook,
            quoteCall {
                quoteConfig: quote.clone(),
            },
            true,
        )
        .await;

    match call_result {
        Ok(v) => {
            if v.typed_return.exists {
                Ok(v.typed_return.into())
            } else {
                Err(FailedQuote::NonExistent)
            }
        }
        Err(e) => Err(e)?,
    }
}

/// quotes array of given orders on a fork using multicall
pub async fn fork_multi_quote(
    quotes: &[Quote],
    orderbook: Address,
    rpc_url: &str,
    fork_block_number: Option<u64>,
    multicall_address: Option<Address>,
) -> Result<Vec<QuoteResult>, ForkCallError> {
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
                calls: quotes
                    .iter()
                    .map(|quote| Call3 {
                        allowFailure: true,
                        target: orderbook,
                        callData: quoteCall {
                            quoteConfig: quote.clone(),
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
                Err(e) => result.push(Err(FailedQuote::ForkCallError(ForkCallError::TypedError(
                    format!(
                        "Call:{:?} Error:{:?} Raw:{:?}",
                        std::any::type_name::<quoteCall>(),
                        e,
                        res.returnData
                    ),
                )))),
            }
        } else {
            match AbiDecodedErrorType::selector_registry_abi_decode(&res.returnData).await {
                Ok(e) => result.push(Err(FailedQuote::ForkCallError(
                    ForkCallError::AbiDecodedError(e),
                ))),
                Err(e) => result.push(Err(FailedQuote::ForkCallError(
                    ForkCallError::AbiDecodeFailed(e),
                ))),
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quote::OrderQuote;
    use alloy_primitives::{hex::decode, U256};
    use rain_orderbook_bindings::IOrderBookV4::{
        addOrder2Call, EvaluableV3, OrderConfigV3, OrderV3, IO,
    };

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
    async fn test_fork_single_quote() {
        let ratio = "f".repeat(64);
        let max_output = "2".repeat(64);
        let block_number = 26938000;
        let (order, orderbook, rpc_url) =
            deploy_order_helper(&ratio, &max_output, block_number).await;

        let quote = Quote {
            order,
            ..Default::default()
        };
        let result = fork_signle_quote(&quote, orderbook, &rpc_url, Some(block_number))
            .await
            .unwrap();
        let expected = OrderQuote {
            max_output: U256::ZERO,
            ratio: U256::MAX,
        };
        assert_eq!(result, expected);

        let quote = Quote {
            ..Default::default()
        };
        let result = fork_signle_quote(&quote, orderbook, &rpc_url, Some(block_number)).await;
        matches!(result, Result::Err(FailedQuote::NonExistent));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fork_multi_quote() {
        let ratio = "f".repeat(64);
        let max_output = "2".repeat(64);
        let block_number = 26938000;
        let (order, orderbook, rpc_url) =
            deploy_order_helper(&ratio, &max_output, block_number).await;

        let quotes = vec![
            Quote {
                order,
                ..Default::default()
            },
            // not exists
            Quote {
                ..Default::default()
            },
        ];
        let result = fork_multi_quote(&quotes, orderbook, &rpc_url, Some(block_number), None)
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
}
