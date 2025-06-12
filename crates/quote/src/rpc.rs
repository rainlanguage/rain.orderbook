use crate::{
    error::{Error, FailedQuote},
    quote::{QuoteResult, QuoteTarget},
};
use alloy::primitives::{hex::FromHex, Address, U256, U64};
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::{
    multicall::{
        IMulticall3::{aggregate3Call, Call3},
        MULTICALL3_ADDRESS,
    },
    transaction::{ReadContractParameters, ReadableClient},
};
use rain_error_decoding::AbiDecodedErrorType;
use rain_orderbook_bindings::IOrderBookV4::quoteCall;

/// Quotes array of given quote targets using the given rpc url
pub async fn batch_quote(
    quote_targets: &[QuoteTarget],
    rpc: &str,
    block_number: Option<u64>,
    gas: Option<U256>,
    multicall_address: Option<Address>,
) -> Result<Vec<QuoteResult>, Error> {
    let client = ReadableClient::new_from_urls(vec![rpc.to_string()])?;
    let parameters = ReadContractParameters {
        gas,
        address: multicall_address.unwrap_or(Address::from_hex(MULTICALL3_ADDRESS).unwrap()),
        block_number: block_number.map(U64::from),
        call: aggregate3Call {
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
        },
    };
    let multicall_result = client.read(parameters).await?;

    let mut result: Vec<QuoteResult> = vec![];
    for res in multicall_result.returnData {
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

#[cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::quote::OrderQuoteValue;
    use alloy::primitives::hex::encode_prefixed;
    use alloy::sol_types::SolValue;
    use alloy_ethers_typecast::multicall::IMulticall3::Result as MulticallResult;
    use alloy_ethers_typecast::transaction::ReadableClientError;
    use alloy_ethers_typecast::{
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

        assert!(
            matches!(
                err,
                Error::RpcCallError(ReadableClientError::CreateReadableClientHttpError(ref msg))
                if msg.contains("No valid providers could be created from the given URLs")
            ),
            "unexpected error: {err}"
        );

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

        assert!(
            matches!(
                err,
                Error::RpcCallError(ReadableClientError::AllProvidersFailed(ref msg))
                if msg.get(rpc_server.url("/rpc").as_str()).is_some()
                    && matches!(
                        msg.get(rpc_server.url("/rpc").as_str()).unwrap(),
                        ReadableClientError::RpcProviderError(_, _)
                    )
            ),
            "unexpected error: {err}"
        );
    }
}
