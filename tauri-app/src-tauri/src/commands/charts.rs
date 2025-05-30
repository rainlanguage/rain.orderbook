use std::collections::HashMap;

use crate::{error::CommandResult, shared_state::SharedState};
use rain_orderbook_common::fuzz::*;
use tauri::State;

#[tauri::command]
pub async fn make_charts(dotrain: String, settings: Option<String>) -> CommandResult<ChartData> {
    let runner = FuzzRunner::new(None);
    let mut context = FuzzRunnerContext::new(&dotrain, settings, None)?;
    Ok(runner.make_chart_data(&mut context).await?)
}

#[tauri::command]
pub async fn make_deployment_debug(
    dotrain: String,
    settings: Option<String>,
    block_numbers: Option<HashMap<u64, u64>>,
    shared_state: State<'_, SharedState>,
) -> CommandResult<DeploymentsDebugDataMap> {
    let mut runner = shared_state.debug_runner.lock().await;
    let mut context = FuzzRunnerContext::new(&dotrain, settings, None)?;
    Ok(runner.make_debug_data(&mut context, block_numbers).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CommandError;
    use alloy::primitives::U256;
    use rain_orderbook_app_settings::{
        plot_source::{
            DotOptionsCfg, HexBinOptionsCfg, HexBinTransformCfg, MarkCfg, TransformCfg,
            TransformOutputsCfg,
        },
        spec_version::SpecVersion,
        yaml::YamlError,
    };
    use rain_orderbook_test_fixtures::LocalEvm;
    use std::str::FromStr;

    fn get_dotrain_prefix(subparser: &str, token1: &str, token2: &str) -> String {
        format!(
            r#"
spec-version: {spec_version}
tokens:
  token1:
    network: "flare"
    address: {token1}
    decimals: 18
  token2:
    network: "flare"
    address: {token2}
    decimals: 18
orders:
  flare:
    orderbook: "flare"
    inputs:
      - token: "token1"
    outputs:
      - token: "token2"
scenarios:
  flare:
    orderbook: "flare"
    runs: 1
    bindings:
      raindex-subparser: {subparser}
      fixed-io-output-token: {token1}
deployments:
  flare:
    order: flare
    scenario: flare
"#,
            spec_version = SpecVersion::current()
        )
    }

    const HAPPY_CHART: &str = r#"
charts:
  flare-chart:
    scenario: "flare"
    plots:
      my-plot:
        title: Test title
        subtitle: Test subtitle
        marks:
          - type: dot
            options:
              x: 1
              y: 2
              r: 3
              fill: "red"
              stroke: "blue"
              transform:
                type: hexbin
                content:
                  outputs:
                    x: 1
                    y: 2
                    r: 3
                    z: 4
                    stroke: "green"
                    fill: "blue"
                  options:
                    x: 1
                    y: 2
                    bin-width: 10
        x:
          label: Test x label
          anchor: start
          label-anchor: middle
          label-arrow: none
        y:
          label: Test y label
          anchor: end
          label-anchor: end
          label-arrow: arrow
        margin: 10
        margin-left: 20
        margin-right: 30
        margin-top: 40
        margin-bottom: 50
        inset: 2
    metrics:
      - label: Metric One
        description: Description for metric one
        value: 42
        precision: 0
      - label: Metric Two
        description: Description for metric two
        unit-prefix: "k"
        value: 3.14
        precision: 2
      - label: Metric Three
        description: Description for metric three
        unit-suffix: "ms"
        value: 100
        precision: 1
      - label: Metric Four
        description: Description for metric four
        unit-prefix: "%"
        value: 87
        precision: 0
"#;

    const RAINLANG: &str = r#"
#raindex-subparser !The subparser to use.
#fixed-io !The io ratio for the limit order.
#fixed-io-output-token !The output token that the fixed io is for. If this doesn't match the runtime output then the fixed-io will be inverted.
#calculate-io
using-words-from raindex-subparser
max-output: max-value(),
io: if(
  equal-to(
    output-token()
    fixed-io-output-token
  )
  fixed-io
  inv(fixed-io)
);
#handle-io
:;
#handle-add-order
:; 
"#;

    fn get_settings(rpc: &str, orderbook: &str, deployer: &str) -> String {
        format!(
            r#"
networks:
  flare:
    rpc: {rpc}
    chain-id: 14
    currency: "FLR"
subgraphs:
  flare: "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/2024-12-13-9dc7/gn"
metaboards:
  flare: "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn"
orderbooks:
  flare:
    address: {orderbook}
    network: "flare"
    subgraph: "flare"
deployers:
  flare:
    address: {deployer}
    network: "flare"
"#
        )
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_make_charts() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let orderbook = &local_evm.orderbook;
        let orderbook_subparser = &local_evm.orderbook_subparser;
        let deployer = &local_evm.deployer;
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let settings = get_settings(
            &local_evm.url(),
            &orderbook.address().to_string(),
            &deployer.address().to_string(),
        );
        let dotrain_prefix = get_dotrain_prefix(
            &orderbook_subparser.address().to_string(),
            &token1.address().to_string(),
            &token2.address().to_string(),
        );

        let dotrain = format!("{}\n{}\n---\n{}", dotrain_prefix, HAPPY_CHART, RAINLANG);
        let res = make_charts(dotrain, Some(settings)).await.unwrap();

        assert_eq!(res.scenarios_data.len(), 1);
        let scenario_data = res.scenarios_data.get("flare").unwrap();
        assert_eq!(scenario_data.scenario, "flare");
        assert_eq!(scenario_data.data.column_names, vec!["0.0", "0.1"]);
        assert_eq!(scenario_data.data.rows.len(), 1);
        assert_eq!(scenario_data.data.rows[0].len(), 2);
        assert_eq!(
            scenario_data.data.rows[0][0],
            U256::from_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .unwrap()
        );
        assert_eq!(scenario_data.data.rows[0][1], U256::from(0));

        assert_eq!(res.charts.len(), 1);
        let chart = res.charts.get("flare-chart").unwrap();
        assert!(chart.plots.is_some());
        assert_eq!(chart.plots.as_ref().unwrap().len(), 1);
        let plot = chart.plots.as_ref().unwrap()[0].clone();
        assert_eq!(plot.title, Some("Test title".to_string()));
        assert_eq!(plot.subtitle, Some("Test subtitle".to_string()));
        assert_eq!(plot.x.unwrap().label, Some("Test x label".to_string()));
        assert_eq!(plot.y.unwrap().label, Some("Test y label".to_string()));
        assert_eq!(plot.margin, Some(10));
        assert_eq!(plot.margin_left, Some(20));
        assert_eq!(plot.margin_right, Some(30));
        assert_eq!(plot.margin_top, Some(40));
        assert_eq!(plot.margin_bottom, Some(50));
        assert_eq!(plot.marks.len(), 1);
        assert_eq!(
            plot.marks[0],
            MarkCfg::Dot(DotOptionsCfg {
                x: Some("1".to_string()),
                y: Some("2".to_string()),
                r: Some(3),
                fill: Some("red".to_string()),
                stroke: Some("blue".to_string()),
                transform: Some(TransformCfg::HexBin(HexBinTransformCfg {
                    outputs: TransformOutputsCfg {
                        x: Some("1".to_string()),
                        y: Some("2".to_string()),
                        r: Some(3),
                        z: Some("4".to_string()),
                        stroke: Some("green".to_string()),
                        fill: Some("blue".to_string()),
                    },
                    options: HexBinOptionsCfg {
                        x: Some("1".to_string()),
                        y: Some("2".to_string()),
                        bin_width: Some(10),
                    },
                })),
            })
        );
        let metrics = chart.metrics.as_ref().unwrap();
        assert_eq!(metrics.len(), 4);
        assert_eq!(metrics[0].label, "Metric One");
        assert_eq!(
            metrics[0].description,
            Some("Description for metric one".to_string())
        );
        assert_eq!(metrics[0].unit_prefix, None);
        assert_eq!(metrics[0].unit_suffix, None);
        assert_eq!(metrics[0].value, "42");
        assert_eq!(metrics[0].precision, Some(0));
        assert_eq!(metrics[1].label, "Metric Two");
        assert_eq!(
            metrics[1].description,
            Some("Description for metric two".to_string())
        );
        assert_eq!(metrics[1].unit_prefix, Some("k".to_string()));
        assert_eq!(metrics[1].unit_suffix, None);
        assert_eq!(metrics[1].value, "3.14");
        assert_eq!(metrics[1].precision, Some(2));
        assert_eq!(metrics[2].label, "Metric Three");
        assert_eq!(
            metrics[2].description,
            Some("Description for metric three".to_string())
        );
        assert_eq!(metrics[2].unit_prefix, None);
        assert_eq!(metrics[2].unit_suffix, Some("ms".to_string()));
        assert_eq!(metrics[2].value, "100");
        assert_eq!(metrics[2].precision, Some(1));
        assert_eq!(metrics[3].label, "Metric Four");
        assert_eq!(
            metrics[3].description,
            Some("Description for metric four".to_string())
        );
        assert_eq!(metrics[3].unit_prefix, Some("%".to_string()));
        assert_eq!(metrics[3].unit_suffix, None);
        assert_eq!(metrics[3].value, "87");
        assert_eq!(metrics[3].precision, Some(0));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_make_charts_missing_scenario() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let orderbook = &local_evm.orderbook;
        let orderbook_subparser = &local_evm.orderbook_subparser;
        let deployer = &local_evm.deployer;
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let settings = get_settings(
            &local_evm.url(),
            &orderbook.address().to_string(),
            &deployer.address().to_string(),
        );
        let dotrain_prefix = get_dotrain_prefix(
            &orderbook_subparser.address().to_string(),
            &token1.address().to_string(),
            &token2.address().to_string(),
        );

        let dotrain = format!(
            "{}\n{}\n---\n{}",
            dotrain_prefix,
            r#"
charts:
  flare-chart:
    scenario: missing
    plots:
"#,
            RAINLANG
        );
        let err = make_charts(dotrain, Some(settings)).await.unwrap_err();

        assert!(
            matches!(
                err,
                CommandError::FuzzRunnerError(FuzzRunnerError::YamlError(YamlError::KeyNotFound(
                    _
                )))
            ),
            "Expected KeyNotFound error, got {:?}",
            err
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_make_charts_rpc_issue() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let orderbook = &local_evm.orderbook;
        let orderbook_subparser = &local_evm.orderbook_subparser;
        let deployer = &local_evm.deployer;
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let settings = get_settings(
            &"https://random.url",
            &orderbook.address().to_string(),
            &deployer.address().to_string(),
        );
        let dotrain_prefix = get_dotrain_prefix(
            &orderbook_subparser.address().to_string(),
            &token1.address().to_string(),
            &token2.address().to_string(),
        );

        let dotrain = format!("{}\n{}\n---\n{}", dotrain_prefix, HAPPY_CHART, RAINLANG);
        let err = make_charts(dotrain, Some(settings)).await.unwrap_err();

        assert!(matches!(
            err,
            CommandError::FuzzRunnerError(FuzzRunnerError::ReadableClientHttpError(
                alloy_ethers_typecast::transaction::ReadableClientError::ReadBlockNumberError(_)
            ))
        ));
    }
}
