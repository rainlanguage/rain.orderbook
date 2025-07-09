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
    rpcs: Vec<String>,
    block_number: Option<u64>,
    _gas: Option<u64>, // TODO: remove or use
    multicall_address: Option<Address>,
) -> Result<Vec<QuoteResult>, Error> {
    let provider = mk_read_provider(rpcs)?;

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
    use alloy::providers::bindings::IMulticall3::Result as MulticallResult;
    use alloy::providers::MulticallError;
    use alloy::sol_types::SolCall;
    use alloy::sol_types::SolValue;
    use alloy::transports::TransportError;
    use httpmock::{Method::POST, MockServer};
    use rain_math_float::Float;
    use rain_orderbook_bindings::IOrderBookV5::{quote2Call, quote2Return};
    use rain_orderbook_common::provider::ReadProviderError;
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
                    outputMax: one.0,
                    ioRatio: two.0,
                })
                .into(),
            },
            MulticallResult {
                success: true,
                returnData: quote2Call::abi_encode_returns(&quote2Return {
                    exists: false,
                    outputMax: zero.0,
                    ioRatio: zero.0,
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
        )
        .await
        .unwrap_err();

        assert!(
            matches!(
                err,
                Error::ReadProviderError(ReadProviderError::UrlParse(
                    url::ParseError::RelativeUrlWithoutBase
                ))
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
}
