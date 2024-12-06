pub mod charts;
pub mod deployment;
pub mod gui;
pub mod order;
pub mod scenarios;

use super::*;
use charts::ChartYaml;
use deployment::DeploymentYaml;
use gui::GuiYaml;
use order::OrderYaml;
use scenarios::ScenarioYaml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DotrainYaml {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub orders: HashMap<String, OrderYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub scenarios: HashMap<String, ScenarioYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub charts: HashMap<String, ChartYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub deployments: HashMap<String, DeploymentYaml>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gui: Option<GuiYaml>,
}

impl DotrainYaml {
    pub fn try_from_string(source: &str) -> Result<Self, YamlError> {
        let mut yaml = Self::default();

        yaml.orders = OrderYaml::try_from_string(source)?;
        yaml.scenarios = ScenarioYaml::try_from_string(source)?;
        yaml.charts = ChartYaml::try_from_string(source)?;
        yaml.deployments = DeploymentYaml::try_from_string(source)?;
        yaml.gui = GuiYaml::try_from_string(source)?;

        Ok(yaml)
    }

    pub fn set_order(&mut self, key: String, order: OrderYaml) {
        self.orders.insert(key, order);
    }

    pub fn set_scenario(&mut self, key: String, scenario: ScenarioYaml) {
        self.scenarios.insert(key, scenario);
    }

    pub fn set_chart(&mut self, key: String, chart: ChartYaml) {
        self.charts.insert(key, chart);
    }

    pub fn set_deployment(&mut self, key: String, deployment: DeploymentYaml) {
        self.deployments.insert(key, deployment);
    }

    pub fn set_gui(&mut self, gui: GuiYaml) {
        self.gui = Some(gui);
    }
}

#[cfg(test)]
mod tests {
    use charts::{
        DotOptions, HexBinOptions, HexBinTransform, LineOptions, Mark, RectYOptions, Transform,
        TransformOutputs,
    };

    use super::*;

    const FULL_YAML: &str = r#"
orders:
    order1:
        inputs:
            - token: eth
              vault-id: 1
        outputs:
            - token: usdc
              vault-id: 2
        deployer: deployer1
        orderbook: orderbook1
scenarios:
    scenario1:
        runs: 10
        blocks: 100
        deployer: deployer1
        bindings:
            key1: value1
            key2: value2
        scenarios:
            scenario2:
                bindings:
                    key3: value3
                scenarios:
                    scenario3:
                        bindings:
                            key4: value4
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                subtitle: this is a subtitle
                marks:
                    - type: dot
                      options:
                        x: 2
                        y: 3
                        r: 4
                        fill: FEDFED
                        stroke: 000000
                        transform:
                            type: hexbin
                            content:
                                outputs:
                                    x: 1
                                    y: 2
                                    r: 3
                                    z: 4
                                    stroke: 000000
                                    fill: FEDFED
                                options:
                                    x: 1
                                    y: 2
                                    bin-width: 3
                    - type: line
                      options:
                        x: 4
                        y: 6
                        r: 8
                        fill: ABABAB
                        stroke: 777777
                        transform:
                            type: hexbin
                            content:
                                outputs:
                                    x: 2
                                    y: 3
                                    r: 4
                                    z: 5
                                    stroke: 777777
                                    fill: ABABAB
                                options:
                                    x: 2
                                    y: 3
                                    bin-width: 4
                    - type: recty
                      options:
                        x0: 4
                        y0: 6
                        x1: 8
                        y1: 10
                x:
                  label: this is x
                  anchor: 1
                  label-anchor: 2
                  label-arrow: 3
                y:
                  label: this is y
                  anchor: 1
                  label-anchor: 2
                  label-arrow: 3
                margin: 10
                margin-left: 20
                margin-right: 30
                margin-top: 40
                margin-bottom: 50
                inset: 60
deployments:
    deployment1:
        scenario: scenario1
        order: order1
gui:
    name: Test GUI
    description: Test Description
    deployments:
        - deployment: deployment1
          name: Test
          description: Test
          deposits:
            - token: eth
              presets:
                - 1.0
                - 2.0   
                - 3.0
            - token: usdc
              presets:
                - 1.0
                - 2.0
                - 3.0
          fields:
            - binding: key1
              name: Field 1
              description: Field 1 description
              presets:
                - value: 1.0
            - binding: key2
              name: Field 2
            - binding: key3
              name: Field 3
            - binding: key4
              name: Field 4
          select-tokens:
            - eth
            - usdc
    "#;

    const YAML_WITHOUT_OPTIONAL_FIELDS: &str = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - token: usdc
scenarios:
    scenario1:
        bindings:
            key1: value1
charts:
    chart1:
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

    #[test]
    fn test_full_yaml() {
        let config = DotrainYaml::try_from_string(FULL_YAML).unwrap();

        assert_eq!(config.orders.len(), 1);
        let order = config.orders.get("order1").unwrap();
        assert_eq!(order.inputs.len(), 1);
        let input = order.inputs.get(0).unwrap();
        assert_eq!(input.token, "eth");
        assert_eq!(input.vault_id, Some("1".to_string()));
        assert_eq!(order.outputs.len(), 1);
        let output = order.outputs.get(0).unwrap();
        assert_eq!(output.token, "usdc");
        assert_eq!(output.vault_id, Some("2".to_string()));
        assert_eq!(order.deployer, Some("deployer1".to_string()));
        assert_eq!(order.orderbook, Some("orderbook1".to_string()));

        assert_eq!(config.scenarios.len(), 1);
        let scenario = config.scenarios.get("scenario1").unwrap();
        assert_eq!(scenario.runs, Some("10".to_string()));
        assert_eq!(scenario.blocks, Some("100".to_string()));
        assert_eq!(scenario.deployer, Some("deployer1".to_string()));
        assert_eq!(scenario.bindings.len(), 2);
        assert_eq!(scenario.bindings.get("key1").unwrap(), "value1");
        assert_eq!(scenario.bindings.get("key2").unwrap(), "value2");
        assert_eq!(scenario.scenarios.is_some(), true);
        let scenario2 = scenario
            .scenarios
            .as_ref()
            .unwrap()
            .get("scenario2")
            .unwrap();
        assert_eq!(scenario2.bindings.get("key3").unwrap(), "value3");
        let scenario3 = scenario2
            .scenarios
            .as_ref()
            .unwrap()
            .get("scenario3")
            .unwrap();
        assert_eq!(scenario3.bindings.get("key4").unwrap(), "value4");

        assert_eq!(config.charts.len(), 1);
        let chart = config.charts.get("chart1").unwrap();
        assert_eq!(chart.plots.is_some(), true);
        assert_eq!(chart.plots.as_ref().unwrap().len(), 1);
        let plot = chart.plots.as_ref().unwrap().get("plot1").unwrap();
        assert_eq!(plot.title, "this is a title".to_string());
        assert_eq!(plot.subtitle, Some("this is a subtitle".to_string()));
        assert_eq!(plot.marks.len(), 3);
        assert_eq!(
            plot.marks[0],
            Mark::Dot(DotOptions {
                x: Some("2".to_string()),
                y: Some("3".to_string()),
                r: Some("4".to_string()),
                fill: Some("FEDFED".to_string()),
                stroke: Some("000000".to_string()),
                transform: Some(Transform::HexBin(HexBinTransform {
                    outputs: TransformOutputs {
                        x: Some("1".to_string()),
                        y: Some("2".to_string()),
                        r: Some("3".to_string()),
                        z: Some("4".to_string()),
                        stroke: Some("000000".to_string()),
                        fill: Some("FEDFED".to_string()),
                    },
                    options: HexBinOptions {
                        x: Some("1".to_string()),
                        y: Some("2".to_string()),
                        bin_width: Some("3".to_string()),
                    },
                })),
            })
        );
        assert_eq!(
            plot.marks[1],
            Mark::Line(LineOptions {
                x: Some("4".to_string()),
                y: Some("6".to_string()),
                r: Some("8".to_string()),
                fill: Some("ABABAB".to_string()),
                stroke: Some("777777".to_string()),
                transform: Some(Transform::HexBin(HexBinTransform {
                    outputs: TransformOutputs {
                        x: Some("2".to_string()),
                        y: Some("3".to_string()),
                        r: Some("4".to_string()),
                        z: Some("5".to_string()),
                        stroke: Some("777777".to_string()),
                        fill: Some("ABABAB".to_string()),
                    },
                    options: HexBinOptions {
                        x: Some("2".to_string()),
                        y: Some("3".to_string()),
                        bin_width: Some("4".to_string()),
                    },
                })),
            })
        );
        assert_eq!(
            plot.marks[2],
            Mark::RectY(RectYOptions {
                x0: Some("4".to_string()),
                y0: Some("6".to_string()),
                x1: Some("8".to_string()),
                y1: Some("10".to_string()),
                transform: None,
            })
        );
        assert_eq!(plot.x.is_some(), true);
        let x = plot.x.as_ref().unwrap();
        assert_eq!(x.label, Some("this is x".to_string()));
        assert_eq!(x.anchor, Some("1".to_string()));
        assert_eq!(x.label_anchor, Some("2".to_string()));
        assert_eq!(x.label_arrow, Some("3".to_string()));
        assert_eq!(plot.y.is_some(), true);
        let y = plot.y.as_ref().unwrap();
        assert_eq!(y.label, Some("this is y".to_string()));
        assert_eq!(y.anchor, Some("1".to_string()));
        assert_eq!(y.label_anchor, Some("2".to_string()));
        assert_eq!(y.label_arrow, Some("3".to_string()));
        assert_eq!(plot.margin, Some("10".to_string()));
        assert_eq!(plot.margin_left, Some("20".to_string()));
        assert_eq!(plot.margin_right, Some("30".to_string()));
        assert_eq!(plot.margin_top, Some("40".to_string()));
        assert_eq!(plot.margin_bottom, Some("50".to_string()));
        assert_eq!(plot.inset, Some("60".to_string()));

        assert_eq!(config.deployments.len(), 1);
        let deployment = config.deployments.get("deployment1").unwrap();
        assert_eq!(deployment.scenario, "scenario1");
        assert_eq!(deployment.order, "order1");

        assert_eq!(config.gui.is_some(), true);
        let gui = config.gui.as_ref().unwrap();
        assert_eq!(gui.name, "Test GUI");
        assert_eq!(gui.description, "Test Description");
        assert_eq!(gui.deployments.len(), 1);
        let deployment = gui.deployments.get(0).unwrap();
        assert_eq!(deployment.deployment, "deployment1");
        assert_eq!(deployment.name, "Test");
        assert_eq!(deployment.description, "Test");
        assert_eq!(deployment.deposits.len(), 2);
        let deposit = deployment.deposits.get(0).unwrap();
        assert_eq!(deposit.token, "eth");
        assert_eq!(deposit.presets.len(), 3);
        assert_eq!(deposit.presets[0], "1.0");
        assert_eq!(deposit.presets[1], "2.0");
        assert_eq!(deposit.presets[2], "3.0");
        let deposit = deployment.deposits.get(1).unwrap();
        assert_eq!(deposit.token, "usdc");
        assert_eq!(deposit.presets.len(), 3);
        assert_eq!(deposit.presets[0], "1.0");
        assert_eq!(deposit.presets[1], "2.0");
        assert_eq!(deposit.presets[2], "3.0");
        assert_eq!(deployment.fields.len(), 4);
        let field = deployment.fields.get(0).unwrap();
        assert_eq!(field.binding, "key1");
        assert_eq!(field.name, "Field 1");
        assert_eq!(field.description, Some("Field 1 description".to_string()));
        assert_eq!(field.presets.is_some(), true);
        let preset = field.presets.as_ref().unwrap().get(0).unwrap();
        assert_eq!(preset.value, "1.0");
        let field = deployment.fields.get(1).unwrap();
        assert_eq!(field.binding, "key2");
        assert_eq!(field.name, "Field 2");
        let field = deployment.fields.get(2).unwrap();
        assert_eq!(field.binding, "key3");
        assert_eq!(field.name, "Field 3");
        let field = deployment.fields.get(3).unwrap();
        assert_eq!(field.binding, "key4");
        assert_eq!(field.name, "Field 4");
    }

    #[test]
    fn test_yaml_without_optional_fields() {
        let config = DotrainYaml::try_from_string(YAML_WITHOUT_OPTIONAL_FIELDS).unwrap();

        assert_eq!(config.orders.len(), 1);
        let order = config.orders.get("order1").unwrap();
        assert_eq!(order.inputs.len(), 1);
        let input = order.inputs.get(0).unwrap();
        assert_eq!(input.token, "eth");
        assert_eq!(order.outputs.len(), 1);
        let output = order.outputs.get(0).unwrap();
        assert_eq!(output.token, "usdc");

        assert_eq!(config.scenarios.len(), 1);
        let scenario = config.scenarios.get("scenario1").unwrap();
        assert_eq!(scenario.bindings.len(), 1);
        assert_eq!(scenario.bindings.get("key1").unwrap(), "value1");

        assert_eq!(config.charts.len(), 1);

        assert_eq!(config.deployments.len(), 1);
        let deployment = config.deployments.get("deployment1").unwrap();
        assert_eq!(deployment.scenario, "scenario1");
        assert_eq!(deployment.order, "order1");

        assert_eq!(config.gui.is_none(), true);
    }
}
