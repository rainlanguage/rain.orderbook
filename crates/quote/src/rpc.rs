use crate::{
    error::{Error, FailedQuote},
    quote::{QuoteResult, QuoteTarget},
};
use alloy::providers::{Failure, Provider};
use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::Address,
};
use rain_error_decoding::AbiDecodedErrorType;
use rain_orderbook_bindings::IOrderBookV5::{quote2Return, IOrderBookV5Instance};
use rain_orderbook_common::provider::mk_read_provider;

/// Quotes array of given quote targets using the given rpc url
pub async fn batch_quote(
    quote_targets: &[QuoteTarget],
    rpc: &str,
    block_number: Option<u64>,
    _gas: Option<u64>, // TODO: remove or use
    multicall_address: Option<Address>,
) -> Result<Vec<QuoteResult>, Error> {
    let provider = mk_read_provider(&[rpc])?;

    let mut multicall = if let Some(addr) = multicall_address {
        provider.multicall().address(addr).dynamic()
    } else {
        provider.multicall().dynamic()
    };

    if let Some(block_number) = block_number {
        multicall = multicall.block(BlockId::Number(BlockNumberOrTag::Number(block_number)));
    }

    for quote_target in quote_targets {
        let ob_instance = IOrderBookV5Instance::new(quote_target.orderbook, provider.clone());
        multicall = multicall.add_dynamic(ob_instance.quote2(quote_target.quote_config.clone()));
    }

    let aggregate_res: Vec<Result<quote2Return, Failure>> = multicall.aggregate3().await?;

    let mut results: Vec<QuoteResult> = Vec::with_capacity(aggregate_res.len());
    for res in aggregate_res {
        match res {
            Ok(ret) => {
                if ret.exists {
                    results.push(Ok(ret.into()));
                } else {
                    results.push(Err(FailedQuote::NonExistent));
                }
            }
            Err(failure) => {
                match AbiDecodedErrorType::selector_registry_abi_decode(&failure.return_data).await
                {
                    Ok(e) => results.push(Err(FailedQuote::RevertError(e))),
                    Err(e) => results.push(Err(FailedQuote::RevertErrorDecodeFailed(e))),
                }
            }
        }
    }

    Ok(results)
}

#[cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::quote::OrderQuoteValue;
    use alloy::primitives::hex::encode_prefixed;
    use alloy::sol_types::SolValue;
    use alloy_ethers_typecast::transaction::multicall::IMulticall3::Result as MulticallResult;
    use alloy_ethers_typecast::transaction::ReadableClientError;
    use alloy_ethers_typecast::transaction::{
        request_shim::{AlloyTransactionRequest, TransactionRequestShim},
        rpc::{eip2718::TypedTransaction, BlockNumber, Request, Response},
    };
    use httpmock::{Method::POST, MockServer};
    use serde_json::{from_str, Value};

    #[tokio::test]
    async fn test_batch_quote_ok() {
        let rpc_server = MockServer::start_async().await;

        let multicall = Address::from_hex(MULTICALL3_ADDRESS).unwrap();

        // build call data
        let quote_targets = vec![
            QuoteTarget::default(),
            QuoteTarget::default(),
            QuoteTarget::default(),
        ];
        let call = aggregate3Call {
            calls: quote_targets
                .iter()
                .map(|quote_target| Call3 {
                    allowFailure: true,
                    target: quote_target.orderbook,
                    callData: quoteCall {
                        quoteConfig: quote_target.quote_config.clone(),
                    }
                    .abi_encode()
                    .into(),
                })
                .collect(),
        };

        // build response data
        let response_data = vec![
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(true, U256::from(1), U256::from(2)))
                    .into(),
            },
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(false, U256::ZERO, U256::ZERO)).into(),
            },
            MulticallResult {
                success: false,
                returnData: vec![].into(),
            },
        ]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/").json_body_partial(
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

        let result = batch_quote(
            &quote_targets,
            rpc_server.url("/").as_str(),
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
        matches!(
            iter_result.next().unwrap(),
            Result::Err(FailedQuote::NonExistent)
        );
        matches!(
            iter_result.next().unwrap(),
            Result::Err(FailedQuote::RevertErrorDecodeFailed(_))
        );
        assert!(iter_result.next().is_none());
    }

    #[tokio::test]
    async fn test_batch_quote_err() {
        let rpc_server = MockServer::start_async().await;
        let quote_targets = vec![QuoteTarget::default()];

        let err = batch_quote(&quote_targets, "this should break", None, None, None)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::RpcCallError(ReadableClientError::CreateReadableClientHttpError(msg))
            if msg == "relative URL without a base"
        ));

        rpc_server.mock(|when, then| {
            when.path("/rpc");
            then.status(500).json_body_obj(
                &from_str::<Value>(
                    &Response::new_error(1, -32000, "Internal error", None)
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        let err = batch_quote(
            &quote_targets,
            rpc_server.url("/rpc").as_str(),
            None,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            Error::RpcCallError(ReadableClientError::AbiDecodedErrorType(
                AbiDecodedErrorType::Unknown(bytestring)
            )) if bytestring.is_empty()
        ));
    }
}
