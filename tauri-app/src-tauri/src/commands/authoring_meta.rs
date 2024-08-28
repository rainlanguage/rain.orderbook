use crate::error::CommandResult;
use alloy::primitives::Address;
use futures::future::join_all;
use rain_orderbook_common::dotrain_order::{AuthoringMetaV2, DotrainOrder};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
pub struct ExtAuthoringMetaV2Word {
    pub word: String,
    pub description: String,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum PragmaResult {
    Success(ExtAuthoringMetaV2),
    Error(String),
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
pub struct PragmaAuthoringMeta {
    address: Address,
    result: PragmaResult,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
pub struct ScenarioPragmas {
    deployer: PragmaAuthoringMeta,
    pragmas: Vec<PragmaAuthoringMeta>,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ScenarioResult {
    Success(ScenarioPragmas),
    Error(String),
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
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

    #[tokio::test]
    async fn test_get_authoring_meta_v2_for_scenarios() {
        let dotrain = format!(
            r#"
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 0
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
scenarios:
    sepolia:
metaboards:
    sepolia: {metaboard_url}
---
#calculate-io
using-words-from 0x8f037f2a3fF2dee510486D9C63A47A245991a4C1
_: order-hash(),
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL,
            metaboard_url = rain_orderbook_env::CI_SEPOLIA_METABOARD_URL,
        );

        let res = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        assert_eq!(res.len(), 1);
        match &res[0].result {
            ScenarioResult::Success(s) => {
                assert_eq!(
                    s.deployer.address,
                    "0x017F5651eB8fa4048BBc17433149c6c035d391A6"
                        .parse::<Address>()
                        .unwrap()
                );
                assert_eq!(s.pragmas.len(), 1);
                assert_eq!(
                    s.pragmas[0].address,
                    "0x8f037f2a3fF2dee510486D9C63A47A245991a4C1"
                        .parse::<Address>()
                        .unwrap()
                );
                match &s.pragmas[0].result {
                    PragmaResult::Success(_) => {}
                    _ => panic!("Expected PragmaResult::Success"),
                }
            }
            _ => panic!("Expected ScenarioResult::Success"),
        }
    }

    #[tokio::test]
    async fn test_get_authoring_meta_v2_for_scenarios_error() {
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
    scenarios:
        sepolia:
    metaboards:
        sepolia: {metaboard_url}
    ---
    #calculate-io
    using-words-from 0x8f037f2a3fF2dee510486D9C63A47A245991a4C1
    _: order-hash()
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL,
            metaboard_url = rain_orderbook_env::CI_SEPOLIA_METABOARD_URL,
        );

        let res = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        assert_eq!(res.len(), 1);
        match &res[0].result {
            ScenarioResult::Error(_) => {}
            _ => panic!("Expected ScenarioResult::Error"),
        }
    }

    #[tokio::test]
    async fn test_get_authoring_meta_v2_for_scenarios_pragma_error() {
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
    scenarios:
        sepolia:
    metaboards:
        sepolia: {metaboard_url}
    ---
    #calculate-io
    using-words-from 0x8f037f2a3fF2dee510486D9C63A47A245991a4C3
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL,
            metaboard_url = rain_orderbook_env::CI_SEPOLIA_METABOARD_URL,
        );

        let res = get_authoring_meta_v2_for_scenarios(dotrain, None)
            .await
            .unwrap();

        assert_eq!(res.len(), 1);

        assert_eq!(res[0].scenario_name, "sepolia".to_string());

        match &res[0].result {
            ScenarioResult::Success(s) => {
                assert_eq!(
                    s.deployer.address,
                    "0x017F5651eB8fa4048BBc17433149c6c035d391A6"
                        .parse::<Address>()
                        .unwrap()
                );
                assert_eq!(s.pragmas.len(), 1);
                assert_eq!(
                    s.pragmas[0].address,
                    "0x8f037f2a3fF2dee510486D9C63A47A245991a4C3"
                        .parse::<Address>()
                        .unwrap()
                );
                match &s.pragmas[0].result {
                    PragmaResult::Error(_) => {}
                    _ => panic!("Expected PragmaResult::Error"),
                }
            }
            ScenarioResult::Error(e) => panic!("Expected ScenarioResult::Success, got: {}", e),
        }
    }
}
