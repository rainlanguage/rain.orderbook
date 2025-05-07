use crate::dotrain_add_order_lsp::LANG_SERVICES;
use dotrain::error::ComposeError;
use dotrain::RainDocument;
use dotrain::Rebind;
use std::collections::HashMap;

/// Compose to rainlang string by setting elided bindings to zero
pub fn compose_to_rainlang(
    dotrain: String,
    bindings: HashMap<String, String>,
    entrypoints: &[&str],
) -> Result<String, ComposeError> {
    let meta_store = LANG_SERVICES.meta_store();

    let rebinds = (!bindings.is_empty()).then_some(
        bindings
            .iter()
            .map(|(k, v)| Rebind(k.clone(), v.clone()))
            .collect(),
    );

    // compose a new RainDocument with final injected bindings
    RainDocument::create(dotrain, Some(meta_store), None, rebinds).compose(entrypoints)
}

#[cfg(test)]
mod tests {
    use dotrain::{error::ErrorCode, types::ast::Problem};

    use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;

    use super::*;

    #[test]
    fn test_compose_to_rainlang_err_empty_entrypoints() {
        let dotrain = r"
some front matter
---
/** this is test */
                                                                           

#const-binding 4e18
#elided-binding ! this elided, rebind before use
#exp-binding
_: opcode-1(0xabcd 456);
"
        .trim_start();

        let bindings = HashMap::new();
        let entrypoints = &[];

        let err = compose_to_rainlang(dotrain.to_string(), bindings, entrypoints).unwrap_err();
        assert_eq!(
            err,
            ComposeError::Reject("no entrypoints specified".to_owned())
        );
    }

    #[test]
    fn test_compose_to_rainlang_ok_empty_bindings() {
        let dotrain = r"
some front matter
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 1 2;
"
        .trim_start();

        let expected = r"
/* 0. calculate-io */ 
_ _: 0 0;

/* 1. handle-io */ 
:;"
        .trim_start();

        let actual = compose_to_rainlang(
            dotrain.to_string(),
            HashMap::new(),
            &ORDERBOOK_ORDER_ENTRYPOINTS,
        )
        .unwrap();

        assert_eq!(actual, expected);

        let expected = r"
/* 0. handle-add-order */ 
_ _: 1 2;"
            .trim_start();

        let actual =
            compose_to_rainlang(dotrain.to_string(), HashMap::new(), &["handle-add-order"])
                .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_compose_to_rainlang_ok_with_bindings() {
        let dotrain = r"
some front matter
---
/** this is test */
                                                                           

#const-binding 4e18
#elided-binding ! this elided, rebind before use
#exp-binding
_ _: const-binding elided-binding;"
            .trim_start();

        let bindings = HashMap::from([(
            "elided-binding".to_string(),
            "0x1234567890abcdef".to_string(),
        )]);

        let expected = r"
/* 0. exp-binding */ 
_ _: 4e18 0x1234567890abcdef;"
            .trim_start();

        let actual = compose_to_rainlang(dotrain.to_string(), bindings, &["exp-binding"]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_compose_to_rainlang_err_with_bindings() {
        let dotrain = r"
some front matter
---
/** this is test */
                                                                           

#const-binding 4e18
#elided-binding ! this elided, rebind before use
#exp-binding
_ _: const-binding elided-binding;"
            .trim_start();

        let bindings = HashMap::new();

        let err = compose_to_rainlang(dotrain.to_string(), bindings, &["exp-binding"]).unwrap_err();

        assert!(matches!(
            err,
            ComposeError::Problems(problems) if problems.len() == 1 && matches!(&problems[0], Problem {
                msg,
                code: ErrorCode::ElidedBinding,
                ..
            } if msg == "elided binding 'elided-binding': this elided, rebind before use")
        ));
    }
}

#[cfg(not(target_family = "wasm"))]
pub use fork_parse::*;

#[cfg(not(target_family = "wasm"))]
mod fork_parse {
    use alloy::primitives::{bytes::Bytes, Address};
    use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
    use once_cell::sync::Lazy;
    use rain_error_decoding::AbiDecodedErrorType;
    use rain_interpreter_eval::error::ForkCallError;
    use rain_interpreter_eval::eval::ForkParseArgs;
    use rain_interpreter_eval::fork::Forker;
    use rain_interpreter_eval::fork::NewForkedEvm;
    use std::sync::Arc;
    use thiserror::Error;
    use tokio::sync::Mutex;

    pub static FORKER: Lazy<Arc<Mutex<Forker>>> = Lazy::new(|| Arc::new(Mutex::new(Forker::new())));

    #[derive(Debug, Error)]
    pub enum ForkParseError {
        #[error("Fork Cache Poisoned")]
        ForkCachePoisoned,
        #[error(transparent)]
        ForkerError(ForkCallError),
        #[error("Fork Call Reverted: {0}")]
        ForkCallReverted(#[from] AbiDecodedErrorType),
        #[error(transparent)]
        ReadableClientError(#[from] ReadableClientError),
        #[error("Failed to read Parser address from deployer")]
        ReadParserAddressFailed,
    }

    impl From<ForkCallError> for ForkParseError {
        fn from(value: ForkCallError) -> Self {
            match value {
                ForkCallError::AbiDecodedError(v) => Self::ForkCallReverted(v),
                other => Self::ForkerError(other),
            }
        }
    }

    /// checks the front matter validity and parses the given rainlang string
    /// with the deployer parsed from the front matter
    /// returns abi encoded expression config on Ok variant
    pub async fn parse_rainlang_on_fork(
        rainlang: &str,
        rpc_url: &str,
        block_number: Option<u64>,
        deployer: Address,
    ) -> Result<Bytes, ForkParseError> {
        // Prepare evm fork
        let block_number_val = match block_number {
            Some(b) => b,
            None => {
                let client = ReadableClientHttp::new_from_url(rpc_url.to_string())?;
                client.get_block_number().await?
            }
        };

        let args = NewForkedEvm {
            fork_url: rpc_url.to_owned(),
            fork_block_number: Some(block_number_val),
        };

        let mut forker = FORKER.lock().await;
        forker.add_or_select(args, None).await?;

        let parse_args = ForkParseArgs {
            rainlang_string: rainlang.to_owned(),
            deployer,
            decode_errors: true,
        };
        let result = forker.fork_parse(parse_args).await?;

        Ok(result.raw.result.0)
    }

    #[cfg(test)]
    mod tests {
        use alloy::primitives::hex::encode_prefixed;
        use httpmock::MockServer;
        use std::str::FromStr;

        use super::*;

        #[tokio::test]
        async fn test_parse_rainlang_on_fork() {
            let mainnet_deployer =
                Address::from_str("0xd19581a021f4704ad4eBfF68258e7A0a9DB1CD77").unwrap();

            let server = MockServer::start();

            server.mock(|when, then| {
                when.path("/rpc").body_contains("eth_gasPrice");

                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{ "jsonrpc": "2.0", "id": 1, "result": "0x198542e9" }"#);
            });

            server.mock(|when, then| {
                when.path("/rpc").body_contains("eth_chainId");

                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{ "jsonrpc": "2.0", "id": 1, "result": "0x1" }"#);
            });

            server.mock(|when, then| {
                when.path("/rpc").body_contains("eth_getBlockByNumber");

                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": {
                            "difficulty": "0x1913ff69551dac",
                            "extraData": "0xe4b883e5bda9e7a59ee4bb99e9b1bc000921",
                            "gasLimit": "0xe4e1b2",
                            "gasUsed": "0xe4d737",
                            "hash": "0xa917fcc721a5465a484e9be17cda0cc5493933dd3bc70c9adbee192cb419c9d7",
                            "logsBloom": "0x00af00124b82093253a6960ab5a003170000318c0a00c18d418505009c10c905810e05d4a4511044b6245a062122010233958626c80039250781851410a468418101040c0100f178088a4e89000140e00001880c1c601413ac47bc5882854701180b9404422202202521584000808843030a552488a80e60c804c8d8004d0480422585320e068028d2e190508130022600024a51c116151a07612040081000088ba5c891064920a846b36288a40280820212b20940280056b233060818988945f33460426105024024040923447ad1102000028b8f0e001e810021031840a2801831a0113b003a5485843004c10c4c10d6a04060a84d88500038ab10875a382c",
                            "miner": "0x829bd824b016326a401d083b33d092293333a830",
                            "mixHash": "0x7d416c4a24dc3b43898040ea788922d8563d44a5193e6c4a1d9c70990775c879",
                            "nonce": "0xe6e41732385c71d6",
                            "number": "0xc5043f",
                            "parentHash": "0xd1c4628a6710d8dec345e5bca6b8093abf3f830516e05e36f419f993334d10ef",
                            "receiptsRoot": "0x7eadd994da137c7720fe2bf2935220409ed23a06ec6470ffd2d478e41af0255b",
                            "sha3Uncles": "0x7d9ce61d799ddcb5dfe1644ec7224ae7018f24ecb682f077b4c477da192e8553",
                            "size": "0xa244",
                            "stateRoot": "0x6350d0454245fb410fc0fb93f6648c5b9047a6081441e36f0ff3ab259c9a47f0",
                            "timestamp": "0x6100bc82",
                            "transactions": [
                                "0x23e3362a76c8b9370dc65bac8eb1cda1d408ac238a466cfe690248025254bf52",
                                "0x4594fadbfa1b5ec0f3a0a13dd1d0ab42d176efd91ef14f6fcb84e9d06b02a159",
                                "0xdf8d8677c9cd5f81d8ee3663a4a64ce7fe93d35fcb46004529e77394630f8e11"
                            ],
                            "transactionsRoot": "0xa17c2a87a6ff2fd790d517e48279e02f2e092a05309300c976363e47e0012672",
                            "uncles": [
                            "0xd3946359c70281162cf00c8164d99ca14801e8008715cb1fad93b9cecaf9f7d8"
                            ]
                        }
                    }"#);
            });

            server.mock(|when, then| {
                when.path("/rpc").matches(|req| {
                    match req.body.as_ref() {
                        Some(body) => {
                            println!("Received request:\n{}\n", String::from_utf8_lossy(body));
                        }
                        None => println!("Received request with no body"),
                    }
                    true
                });

                then.status(500);
            });

            let block_number = 12911679;

            let rainlang = r"
some front matter
---
/** this is test */

#const-binding 4e18
#elided-binding ! this elided, rebind before use
#exp-binding
_: opcode-1(0xabcd 456);
";

            let bytes = parse_rainlang_on_fork(
                rainlang,
                &server.url("/rpc"),
                Some(block_number),
                mainnet_deployer,
            )
            .await
            .unwrap();

            let expected = "0x";

            assert_eq!(encode_prefixed(&bytes), expected);
        }
    }
}
