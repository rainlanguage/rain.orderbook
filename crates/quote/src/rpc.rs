use crate::{
    error::{Error, FailedQuote},
    quote::{QuoteResult, QuoteTarget},
};
use alloy::providers::{MulticallError, Provider};
use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::Address,
};
use rain_error_decoding::{AbiDecodedErrorType, ErrorRegistry};
use rain_orderbook_bindings::provider::mk_read_provider;
use rain_orderbook_bindings::IOrderBookV5::IOrderBookV5Instance;
use url::Url;

/// Quotes array of given quote targets using the given rpc url
pub async fn batch_quote(
    quote_targets: &[QuoteTarget],
    rpcs: Vec<String>,
    block_number: Option<u64>,
    _gas: Option<u64>, // TODO: remove or use
    multicall_address: Option<Address>,
    registry: Option<&dyn ErrorRegistry>,
) -> Result<Vec<QuoteResult>, Error> {
    let rpcs = rpcs
        .into_iter()
        .map(|rpc| rpc.parse::<Url>())
        .collect::<Result<Vec<Url>, _>>()?;

    let provider = mk_read_provider(&rpcs)?;

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

    let aggregate_res = match multicall.aggregate3().await {
        Ok(results) => results,
        Err(MulticallError::CallFailed(bytes)) => {
            let decoded_error =
                match AbiDecodedErrorType::selector_registry_abi_decode(bytes.as_ref(), registry)
                    .await
                {
                    Ok(err) => FailedQuote::RevertError(err),
                    Err(err) => FailedQuote::RevertErrorDecodeFailed(err),
                };
            return Ok((0..quote_targets.len())
                .map(|_| match &decoded_error {
                    FailedQuote::RevertError(abi_err) => {
                        Err(FailedQuote::RevertError(abi_err.clone()))
                    }
                    FailedQuote::RevertErrorDecodeFailed(_) => Err(FailedQuote::CorruptReturnData(
                        "Multicall failed with non-decodable error".to_string(),
                    )),
                    _ => Err(FailedQuote::CorruptReturnData(
                        "Unexpected multicall failure".to_string(),
                    )),
                })
                .collect());
        }
        Err(err) => return Err(Error::MulticallError(err)),
    };

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
                match AbiDecodedErrorType::selector_registry_abi_decode(
                    &failure.return_data,
                    registry,
                )
                .await
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
    use alloy::json_abi::Error as AlloyError;
    use alloy::providers::bindings::IMulticall3::Result as MulticallResult;
    use alloy::providers::MulticallError;
    use alloy::sol_types::SolCall;
    use alloy::sol_types::SolValue;
    use alloy::transports::TransportError;
    use httpmock::{Method::POST, MockServer};
    use rain_error_decoding::ErrorRegistry;
    use rain_math_float::Float;
    use rain_orderbook_bindings::IOrderBookV5::{quote2Call, quote2Return};
    use serde_json::json;

    #[tokio::test]
    async fn test_batch_quote_ok() {
        let rpc_server = MockServer::start_async().await;

        let zero = Float::parse("0".to_string()).unwrap();
        let one = Float::parse("1".to_string()).unwrap();
        let two = Float::parse("2".to_string()).unwrap();

        // build call data
        let quote_targets = vec![
            QuoteTarget::default(),
            QuoteTarget::default(),
            QuoteTarget::default(),
        ];

        // build response data
        let response_data = vec![
            MulticallResult {
                success: true,
                returnData: quote2Call::abi_encode_returns(&quote2Return {
                    exists: true,
                    outputMax: one.get_inner(),
                    ioRatio: two.get_inner(),
                })
                .into(),
            },
            MulticallResult {
                success: true,
                returnData: quote2Call::abi_encode_returns(&quote2Return {
                    exists: false,
                    outputMax: zero.get_inner(),
                    ioRatio: zero.get_inner(),
                })
                .into(),
            },
            MulticallResult {
                success: false,
                returnData: vec![].into(),
            },
        ]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": alloy::hex::encode_prefixed(response_data).as_str(),
            }));
        });

        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/").to_string()],
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 3);

        assert!(result[0].as_ref().unwrap().max_output.eq(one).unwrap());
        assert!(result[0].as_ref().unwrap().ratio.eq(two).unwrap());

        assert!(matches!(result[1], Err(FailedQuote::NonExistent)));
        assert!(matches!(
            result[2],
            Err(FailedQuote::RevertErrorDecodeFailed(_))
        ));
    }

    #[tokio::test]
    async fn test_batch_quote_err() {
        let rpc_server = MockServer::start_async().await;
        let quote_targets = vec![QuoteTarget::default()];

        let err = batch_quote(
            &quote_targets,
            vec!["this should break".to_string()],
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert!(
            matches!(
                err,
                Error::UrlParseError(url::ParseError::RelativeUrlWithoutBase)
            ),
            "unexpected error: {err:?}"
        );

        rpc_server.mock(|when, then| {
            when.path("/rpc");
            then.status(500).json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32000,
                    "message": "Internal error"
                }
            }));
        });

        let err = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert!(
            matches!(
                err,
                Error::MulticallError(MulticallError::TransportError(TransportError::Transport(_)))
            ),
            "unexpected error: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_batch_quote_handles_individual_call_failures_with_rain_error() {
        let rpc_server = MockServer::start_async().await;
        let quote_targets = vec![QuoteTarget::default(), QuoteTarget::default()];

        let response_data = vec![
            MulticallResult {
                success: false,
                returnData: alloy::hex!("734bc71c").to_vec().into(), // TokenSelfTrade error selector
            },
            MulticallResult {
                success: false,
                returnData: alloy::hex!("ff00ff00").to_vec().into(), // Unknown error selector
            },
        ]
        .abi_encode();

        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": alloy::hex::encode_prefixed(response_data),
            }));
        });

        struct FakeRegistry;

        #[async_trait::async_trait]
        impl ErrorRegistry for FakeRegistry {
            async fn lookup(
                &self,
                selector: [u8; 4],
            ) -> Result<Vec<AlloyError>, rain_error_decoding::AbiDecodeFailedErrors> {
                // 0x734bc71c -> "TokenSelfTrade()"
                if selector == [0x73, 0x4b, 0xc7, 0x1c] {
                    Ok(vec!["TokenSelfTrade()".parse().unwrap()])
                } else {
                    Ok(vec![]) // keep 0xdeadbeef unknown
                }
            }
        }

        let results = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            None,
            None,
            Some(&FakeRegistry),
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);

        assert!(matches!(
            &results[0],
            Err(FailedQuote::RevertError(rain_error_decoding::AbiDecodedErrorType::Known { name, .. }))
            if name == "TokenSelfTrade"
        ));

        assert!(matches!(
            &results[1],
            Err(FailedQuote::RevertError(
                rain_error_decoding::AbiDecodedErrorType::Unknown(_)
            ))
        ));
    }
}
