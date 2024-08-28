use crate::error::CommandResult;
use alloy::primitives::Address;
use futures::future::join_all;
use rain_orderbook_common::dotrain_order::{AuthoringMetaV2, DotrainOrder};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ExtAuthoringMetaV2Word {
    pub word: String,
    pub description: String,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ExtAuthoringMetaV2 {
    pub words: Vec<ExtAuthoringMetaV2Word>,
}

impl From<AuthoringMetaV2> for ExtAuthoringMetaV2 {
    fn from(authoring_meta: AuthoringMetaV2) -> Self {
        let words = authoring_meta
            .words
            .into_iter()
            .map(|word| ExtAuthoringMetaV2Word {
                word: word.word,
                description: word.description,
            })
            .collect();
        Self { words }
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum PragmaResult {
    Success(ExtAuthoringMetaV2),
    Error(String),
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PragmaAuthoringMeta {
    address: Address,
    result: PragmaResult,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ScenarioPragmas {
    deployer: PragmaAuthoringMeta,
    pragmas: Vec<PragmaAuthoringMeta>,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ScenarioResult {
    Success(ScenarioPragmas),
    Error(String),
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ScenarioAuthoringMeta {
    scenario_name: String,
    result: ScenarioResult,
}

#[tauri::command]
pub async fn get_authoring_meta_v2_for_scenarios(
    dotrain: String,
    settings: Option<String>,
) -> CommandResult<Vec<ScenarioAuthoringMeta>> {
    let order = DotrainOrder::new(dotrain, settings).await?;
    let mut futures = vec![];
    let scenarios_keys: Vec<&String> = order.config.scenarios.keys().collect();
    for scenario in &scenarios_keys {
        futures.push(order.get_scenario_all_words(scenario))
    }

    let results = join_all(futures).await;
    results
        .into_iter()
        .enumerate()
        .map(|(i, result)| match result {
            Err(e) => Ok(ScenarioAuthoringMeta {
                scenario_name: scenarios_keys[i].clone(),
                result: ScenarioResult::Error(e.to_string()),
            }),
            Ok((addresses, meta_results)) => {
                let mut pragma_results = vec![];
                let deployer_result = match &meta_results[0] {
                    Ok(meta) => PragmaAuthoringMeta {
                        address: addresses[0],
                        result: PragmaResult::Success(meta.clone().into()),
                    },
                    Err(e) => PragmaAuthoringMeta {
                        address: addresses[0],
                        result: PragmaResult::Error(e.to_string()),
                    },
                };
                for (j, item) in meta_results.into_iter().enumerate().skip(1) {
                    pragma_results.push(match item {
                        Ok(meta) => PragmaAuthoringMeta {
                            address: addresses[j],
                            result: PragmaResult::Success(meta.into()),
                        },
                        Err(e) => PragmaAuthoringMeta {
                            address: addresses[j],
                            result: PragmaResult::Error(e.to_string()),
                        },
                    })
                }
                Ok(ScenarioAuthoringMeta {
                    scenario_name: scenarios_keys[i].clone(),
                    result: ScenarioResult::Success(ScenarioPragmas {
                        deployer: deployer_result,
                        pragmas: pragma_results,
                    }),
                })
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{hex::encode_prefixed, primitives::B256, sol, sol_types::SolValue};
    use alloy_ethers_typecast::rpc::Response;
    use httpmock::MockServer;
    use rain_metadata::{KnownMagic, RainMetaDocumentV1Item};
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
        let pragma_addresses = vec![Address::random()];
        let deployer_address = Address::random();
        let server = mock_server(pragma_addresses.clone());
        let dotrain = format!(
            r#"
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 0
deployers:
    sepolia:
        address: {deployer}
scenarios:
    sepolia:
metaboards:
    sepolia: {metaboard_url}
---
#calculate-io
using-words-from {pragma}
_: order-hash(),
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
            pragma = encode_prefixed(pragma_addresses[0]),
            deployer = encode_prefixed(deployer_address)
        );

        let res = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        let expected = vec![ScenarioAuthoringMeta {
            scenario_name: "sepolia".to_string(),
            result: ScenarioResult::Success(ScenarioPragmas {
                deployer: PragmaAuthoringMeta {
                    address: deployer_address,
                    result: PragmaResult::Success(ExtAuthoringMetaV2 {
                        words: vec![
                            ExtAuthoringMetaV2Word {
                                word: "some-word".to_string(),
                                description: "some-desc".to_string(),
                            },
                            ExtAuthoringMetaV2Word {
                                word: "some-other-word".to_string(),
                                description: "some-other-desc".to_string(),
                            },
                        ],
                    }),
                },
                pragmas: vec![PragmaAuthoringMeta {
                    address: pragma_addresses[0],
                    result: PragmaResult::Success(ExtAuthoringMetaV2 {
                        words: vec![
                            ExtAuthoringMetaV2Word {
                                word: "some-word".to_string(),
                                description: "some-desc".to_string(),
                            },
                            ExtAuthoringMetaV2Word {
                                word: "some-other-word".to_string(),
                                description: "some-other-desc".to_string(),
                            },
                        ],
                    }),
                }],
            }),
        }];
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_get_authoring_meta_v2_for_scenarios_error() {
        let pragma_addresses = [Address::random()];
        let deployer_address = Address::random();
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: {deployer}
    scenarios:
        sepolia:
    metaboards:
        sepolia: {metaboard_url}
    ---
    #calculate-io
    using-words-from {pragma}
    _: order-hash()
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/bad-sg"),
            pragma = encode_prefixed(pragma_addresses[0]),
            deployer = encode_prefixed(deployer_address)
        );

        let res = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        assert_eq!(res.len(), 1);
        matches!(
            res[0],
            ScenarioAuthoringMeta {
                result: ScenarioResult::Error(_),
                ..
            }
        );
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
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: {deployer}
    scenarios:
        sepolia:
    metaboards:
        sepolia: {metaboard_url}
    ---
    #calculate-io
    using-words-from {pragma}
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
            pragma = encode_prefixed(pragma_addresses[0]),
            deployer = encode_prefixed(deployer_address)
        );

        let res = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();
        let expected = vec![
            ScenarioAuthoringMeta {
                scenario_name: "sepolia".to_string(),
                result: ScenarioResult::Success(ScenarioPragmas {
                    deployer: PragmaAuthoringMeta {
                        address: deployer_address,
                        result: PragmaResult::Success(ExtAuthoringMetaV2 {
                            words: vec![
                                ExtAuthoringMetaV2Word {
                                    word: "some-word".to_string(),
                                    description: "some-desc".to_string()
                                },
                                ExtAuthoringMetaV2Word {
                                    word: "some-other-word".to_string(),
                                    description: "some-other-desc".to_string()
                                }
                            ]
                        })
                    },
                    pragmas: vec![
                        PragmaAuthoringMeta {
                            address: pragma_addresses[0],
                            result: PragmaResult::Error(format!(
                                "Error fetching authoring meta for contract {}, RPC URL {}, Metaboard URL {}: Subgraph query returned no data for metahash {}",
                                pragma_addresses[0],
                                server.url("/rpc"),
                                server.url("/sg"),
                                pragma_meta_hash,
                            ))
                        },
                    ]
                })
            }
        ];
        assert_eq!(res, expected);
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
