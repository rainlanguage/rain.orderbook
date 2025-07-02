use crate::error::CommandResult;
use rain_orderbook_common::dotrain_order::{DotrainOrder, ScenarioWords};

#[tauri::command]
pub async fn get_authoring_meta_v2_for_scenarios(
    dotrain: String,
    settings: Option<Vec<String>>,
) -> CommandResult<Vec<ScenarioWords>> {
    let dotrain_order = DotrainOrder::create(dotrain, settings).await?;
    Ok(dotrain_order.get_all_scenarios_all_words().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex::encode_prefixed,
        primitives::{Address, B256},
        sol,
        sol_types::SolValue,
    };
    use alloy_ethers_typecast::rpc::Response;
    use httpmock::MockServer;
    use rain_metadata::{KnownMagic, RainMetaDocumentV1Item};
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_common::dotrain_order::WordsResult;
    use serde_bytes::ByteBuf;

    sol!(
        struct AuthoringMetaV2Sol {
            bytes32 word;
            string description;
        }
    );
    sol!(
        struct PragmaV1 { address[] usingWordsFrom; }
    );

    #[tokio::test]
    async fn test_get_authoring_meta_v2_for_scenarios_happy() {
        let deployer_address = Address::random();
        let pragma_addresses = vec![Address::random()];
        let server = mock_server(pragma_addresses.clone());
        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    sepolia:
        rpcs:
            - {rpc_url}
        chain-id: 0
deployers:
    sepolia:
        address: {deployer}
scenarios:
    sepolia:
        deployer: sepolia
        bindings:
            key: 10
metaboards:
    sepolia: {metaboard_url}
---
#key !Test binding
#calculate-io
using-words-from {pragma}
_: order-hash(),
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
            pragma = encode_prefixed(pragma_addresses[0]),
            deployer = encode_prefixed(deployer_address),
            spec_version = SpecVersion::current()
        );

        let results = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        for result in results {
            assert_eq!(&result.scenario, "sepolia");

            assert_eq!(result.deployer_words.address, deployer_address);
            assert!(matches!(
                result.deployer_words.words,
                WordsResult::Success(_)
            ));
            if let WordsResult::Success(authoring_meta) = &result.deployer_words.words {
                assert_eq!(&authoring_meta.words[0].word, "some-word");
                assert_eq!(&authoring_meta.words[0].description, "some-desc");

                assert_eq!(&authoring_meta.words[1].word, "some-other-word");
                assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
            }

            assert!(result.pragma_words.len() == 1);
            assert_eq!(result.pragma_words[0].address, pragma_addresses[0]);
            assert!(matches!(
                result.pragma_words[0].words,
                WordsResult::Success(_)
            ));
            if let WordsResult::Success(authoring_meta) = &result.pragma_words[0].words {
                assert_eq!(&authoring_meta.words[0].word, "some-word");
                assert_eq!(&authoring_meta.words[0].description, "some-desc");

                assert_eq!(&authoring_meta.words[1].word, "some-other-word");
                assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
            }
        }
    }

    #[tokio::test]
    async fn test_get_authoring_meta_v2_for_scenarios_error() {
        let pragma_addresses = [Address::random()];
        let deployer_address = Address::random();
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
    version: {spec_version}
    networks:
        sepolia:
            rpcs:
                - {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: {deployer}
    scenarios:
        sepolia:
            deployer: sepolia
            bindings:
                key: 10
    metaboards:
        sepolia: {metaboard_url}
    ---
    #key !Test binding
    #calculate-io
    using-words-from {pragma}
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/bad-sg"),
            pragma = encode_prefixed(pragma_addresses[0]),
            deployer = encode_prefixed(deployer_address),
            spec_version = SpecVersion::current()
        );

        let results = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        for result in results {
            assert_eq!(&result.scenario, "sepolia");
            assert!(&result.pragma_words.is_empty());
            assert_eq!(result.deployer_words.address, deployer_address);
            assert!(matches!(result.deployer_words.words, WordsResult::Error(_)));
        }
    }

    #[tokio::test]
    async fn test_get_authoring_meta_v2_for_scenarios_pragma_error() {
        let server = MockServer::start();

        let deployer_address = Address::random();
        let pragma_addresses = vec![Address::random()];
        let pragma_meta_hash = encode_prefixed(B256::random());
        let deployer_meta_hash = encode_prefixed(B256::random());

        // mock shared contract calls
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7ffffffff");
            then.body(
                Response::new_success(1, &B256::left_padding_from(&[0]).to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7");
            then.body(
                Response::new_success(1, &B256::left_padding_from(&[1]).to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x5514ca20");
            then.body(
                Response::new_success(
                    1,
                    &encode_prefixed(
                        PragmaV1 {
                            usingWordsFrom: pragma_addresses.clone(),
                        }
                        .abi_encode(),
                    ),
                )
                .to_json_string()
                .unwrap(),
            );
        });

        // mock contract and sg calls for deployer
        server.mock(|when, then| {
            when.path("/rpc").json_body_partial(format!(
                "{{\"params\":[{{\"to\":\"{}\",\"data\":\"0x6f5aa28d\"}}]}}",
                encode_prefixed(deployer_address)
            ));
            then.body(
                Response::new_success(1, &deployer_meta_hash)
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/sg").body_contains(&deployer_meta_hash);
            then.status(200).json_body_obj(&get_sg_mocked_meta());
        });

        // mock contract and sg calls for pragma
        server.mock(|when, then| {
            when.path("/rpc").json_body_partial(format!(
                "{{\"params\":[{{\"to\":\"{}\",\"data\":\"0x6f5aa28d\"}}]}}",
                encode_prefixed(pragma_addresses[0])
            ));
            then.body(
                Response::new_success(1, &pragma_meta_hash)
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/sg").body_contains(&pragma_meta_hash);
            then.status(200)
                .json_body_obj(&serde_json::json!({"data": {"metaV1S": []}}));
        });

        let dotrain = format!(
            r#"
    version: {spec_version}
    networks:
        sepolia:
            rpcs:
                - {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: {deployer}
    scenarios:
        sepolia:
            deployer: sepolia
            bindings:
                key: 10
    metaboards:
        sepolia: {metaboard_url}
    ---
    #key !Test binding
    #calculate-io
    using-words-from {pragma}
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
            pragma = encode_prefixed(pragma_addresses[0]),
            deployer = encode_prefixed(deployer_address),
            spec_version = SpecVersion::current()
        );

        let results = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        for result in results {
            assert_eq!(&result.scenario, "sepolia");

            assert_eq!(result.deployer_words.address, deployer_address);
            assert!(matches!(
                result.deployer_words.words,
                WordsResult::Success(_)
            ));
            if let WordsResult::Success(authoring_meta) = &result.deployer_words.words {
                assert_eq!(&authoring_meta.words[0].word, "some-word");
                assert_eq!(&authoring_meta.words[0].description, "some-desc");

                assert_eq!(&authoring_meta.words[1].word, "some-other-word");
                assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
            }

            assert!(result.pragma_words.len() == 1);
            assert_eq!(result.pragma_words[0].address, pragma_addresses[0]);
            assert!(matches!(
                result.pragma_words[0].words,
                WordsResult::Error(_)
            ));
            if let WordsResult::Error(e) = &result.pragma_words[0].words {
                assert_eq!(
                    e,
                    &format!(
                        "Error fetching authoring meta for contract {}, RPCs {:?}, Metaboard URL {}: Subgraph query returned no data for metahash {}",
                        pragma_addresses[0],
                        vec![server.url("/rpc")],
                        server.url("/sg"),
                        pragma_meta_hash,
                    )
                );
            }
        }
    }

    fn get_sg_mocked_meta() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "metaV1S": [{
                    "meta": encode_prefixed(
                        RainMetaDocumentV1Item {
                            payload: ByteBuf::from(
                                vec![
                                    AuthoringMetaV2Sol {
                                        word: B256::right_padding_from(
                                            "some-word".as_bytes()
                                        ),
                                        description: "some-desc".to_string(),
                                    },
                                    AuthoringMetaV2Sol {
                                        word: B256::right_padding_from(
                                            "some-other-word".as_bytes()
                                        ),
                                        description: "some-other-desc".to_string(),
                                    }
                                ]
                                .abi_encode(),
                            ),
                            magic: KnownMagic::AuthoringMetaV2,
                            content_type: rain_metadata::ContentType::OctetStream,
                            content_encoding: rain_metadata::ContentEncoding::None,
                            content_language: rain_metadata::ContentLanguage::None,
                        }
                        .cbor_encode()
                        .unwrap()
                    ),
                    "metaHash": "0x00",
                    "sender": "0x00",
                    "id": "0x00",
                    "metaBoard": {
                        "id": "0x00",
                        "metas": [],
                        "address": "0x00",
                    },
                    "subject": "0x00",
                }]
            }
        })
    }

    // helper function to mock rpc and sg response
    fn mock_server(with_pragma_addresses: Vec<Address>) -> MockServer {
        let server = MockServer::start();
        // mock contract calls
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7ffffffff");
            then.body(
                Response::new_success(1, &B256::left_padding_from(&[0]).to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7");
            then.body(
                Response::new_success(1, &B256::left_padding_from(&[1]).to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x6f5aa28d");
            then.body(
                Response::new_success(1, &B256::random().to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x5514ca20");
            then.body(
                Response::new_success(
                    1,
                    &encode_prefixed(
                        PragmaV1 {
                            usingWordsFrom: with_pragma_addresses,
                        }
                        .abi_encode(),
                    ),
                )
                .to_json_string()
                .unwrap(),
            );
        });

        // mock sg query
        server.mock(|when, then| {
            when.path("/sg");
            then.status(200).json_body_obj(&get_sg_mocked_meta());
        });
        server
    }
}
