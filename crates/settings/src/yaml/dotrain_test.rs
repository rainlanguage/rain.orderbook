#[cfg(test)]
mod tests {
    use crate::yaml::YamlError;

    use super::super::dotrain::*;

    const FULL_VALID_YAML: &str = r#"
orders:
    order1:
        inputs:
            - token: eth
              vault-id: "1"
        outputs:
            - token: usdc
              vault-id: "2"
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
        scenario: scenario1
        plots:
            plot1: plot value
            plot2: plot value
        metrics:
            - label: "Metric 1"
              description: "Metric 1 description"
              unit_prefix: "Unit prefix"
              unit_suffix: "Unit suffix"
              value: "100"
              precision: "2"
deployments:
    deployment1:
        scenario: scenario1
        order: order1
gui:
    name: "Test GUI"
    description: "Test Description"
    deployments:
        - deployment: deployment1
          name: "Test"
          description: "Test"
          deposits:
            - token: eth
              presets:
                - "1.0"
                - "2.0"
                - "3.0"
            - token: usdc
              presets:
                - "1.0"
                - "2.0"
                - "3.0"
          fields:
            - binding: key1
              name: "Field 1"
              description: "Field 1 description"
              presets:
                - value: "1.0"
            - binding: key2
              name: "Field 2"
            - binding: key3
              name: "Field 3"
            - binding: key4
              name: "Field 4"
          select-tokens:
            - eth
            - usdc
"#;

    const VALID_YAML_WITHOUT_OPTIONAL_FIELDS: &str = r#"
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
    fn test_valid_yaml() {
        let config = DotrainYaml::new(FULL_VALID_YAML).unwrap();
        assert_eq!(config.orders.len(), 1);
        let order = config.orders.get("order1").unwrap();
        assert_eq!(order.inputs.len(), 1);
        assert_eq!(order.inputs[0].token, "eth");
        assert_eq!(order.inputs[0].vault_id, Some("1".to_string()));
        assert_eq!(order.outputs.len(), 1);
        assert_eq!(order.outputs[0].token, "usdc");
        assert_eq!(order.outputs[0].vault_id, Some("2".to_string()));
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
        assert_eq!(scenario.clone().scenarios.unwrap().len(), 1);
        let sub_scenario = scenario
            .scenarios
            .as_ref()
            .unwrap()
            .get("scenario2")
            .unwrap();
        assert_eq!(sub_scenario.bindings.len(), 1);
        assert_eq!(sub_scenario.bindings.get("key3").unwrap(), "value3");
        assert_eq!(sub_scenario.scenarios.as_ref().unwrap().len(), 1);
        let sub_sub_scenario = sub_scenario
            .scenarios
            .as_ref()
            .unwrap()
            .get("scenario3")
            .unwrap();
        assert_eq!(sub_sub_scenario.bindings.len(), 1);
        assert_eq!(sub_sub_scenario.bindings.get("key4").unwrap(), "value4");
        assert_eq!(config.charts.len(), 1);
        let chart = config.charts.get("chart1").unwrap();
        assert_eq!(chart.scenario, Some("scenario1".to_string()));
        // assert_eq!(chart.plots.as_ref().unwrap().len(), 1);
        // assert_eq!(
        //     chart.plots.as_ref().unwrap().get("label"),
        //     Some(&"Plot 1".to_string())
        // );
        // assert_eq!(
        //     chart.plots.as_ref().unwrap().get("value"),
        //     Some(&"plot1".to_string())
        // );
        assert_eq!(chart.metrics.as_ref().unwrap().len(), 1);
        let metric = chart.metrics.as_ref().unwrap()[0].clone();
        assert_eq!(metric.label, "Metric 1");
        assert_eq!(metric.description.unwrap(), "Metric 1 description");
        assert_eq!(metric.unit_prefix.unwrap(), "Unit prefix");
        assert_eq!(metric.unit_suffix.unwrap(), "Unit suffix");
        assert_eq!(metric.value, "100");
        assert_eq!(metric.precision.unwrap(), "2");
        assert_eq!(config.deployments.len(), 1);
        let deployment = config.deployments.get("deployment1").unwrap();
        assert_eq!(deployment.scenario, "scenario1");
        assert_eq!(deployment.order, "order1");
        let gui = config.gui.as_ref().unwrap();
        assert_eq!(gui.name, "Test GUI");
        assert_eq!(gui.description, "Test Description");
        assert_eq!(gui.deployments.len(), 1);
        let gui_deployment = gui.deployments[0].clone();
        assert_eq!(gui_deployment.deployment, "deployment1");
        assert_eq!(gui_deployment.name, "Test");
        assert_eq!(gui_deployment.description, "Test");
        assert_eq!(gui_deployment.deposits.len(), 2);
        assert_eq!(gui_deployment.deposits[0].token, "eth");
        assert_eq!(gui_deployment.deposits[0].presets.len(), 3);
        assert_eq!(gui_deployment.deposits[0].presets[0], "1.0");
        assert_eq!(gui_deployment.deposits[0].presets[1], "2.0");
        assert_eq!(gui_deployment.deposits[0].presets[2], "3.0");
        assert_eq!(gui_deployment.deposits[1].token, "usdc");
        assert_eq!(gui_deployment.deposits[1].presets.len(), 3);
        assert_eq!(gui_deployment.deposits[1].presets[0], "1.0");
        assert_eq!(gui_deployment.deposits[1].presets[1], "2.0");
        assert_eq!(gui_deployment.deposits[1].presets[2], "3.0");
        assert_eq!(gui_deployment.fields.len(), 4);
        assert_eq!(gui_deployment.fields[0].binding, "key1");
        assert_eq!(gui_deployment.fields[0].name, "Field 1");
        assert_eq!(
            gui_deployment.fields[0].description.as_ref().unwrap(),
            "Field 1 description"
        );
        assert_eq!(gui_deployment.fields[1].binding, "key2");
        assert_eq!(gui_deployment.fields[2].binding, "key3");
        assert_eq!(gui_deployment.fields[3].binding, "key4");
        assert_eq!(gui_deployment.select_tokens.as_ref().unwrap().len(), 2);
        assert_eq!(gui_deployment.select_tokens.as_ref().unwrap()[0], "eth");
        assert_eq!(gui_deployment.select_tokens.as_ref().unwrap()[1], "usdc");
    }

    #[test]
    fn test_valid_yaml_without_optional_fields() {
        let config = DotrainYaml::new(VALID_YAML_WITHOUT_OPTIONAL_FIELDS).unwrap();
        assert_eq!(config.orders.len(), 1);
        assert_eq!(config.scenarios.len(), 1);
        assert_eq!(config.charts.len(), 1);
        assert_eq!(config.deployments.len(), 1);
    }

    #[test]
    fn test_orders() {
        let yaml = r#"
test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field orders".to_string())
        );

        let yaml = r#"
orders:
    order1:
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("inputs missing for order \"order1\"".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token missing in input index 0 for order \"order1\"".to_string()
            )
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("outputs missing for order \"order1\"".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token missing in output index 0 for order \"order1\"".to_string()
            )
        );
    }

    #[test]
    fn test_scenarios() {
        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - token: usdc
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field scenarios".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - token: usdc
scenarios:
    scenario1:
        test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("bindings missing for scenario \"scenario1\"".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - token: usdc
scenarios:
    scenario1:
        bindings:
            key1:
              - value1
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("binding value must be a string for key \"key1\"".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - token: usdc
scenarios:
    scenario1:
        bindings:
            key1:
              - value1: value2
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("binding value must be a string for key \"key1\"".to_string())
        );
    }

    #[test]
    fn test_charts() {
        let yaml = r#"
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
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field charts".to_string())
        );

        let yaml = r#"
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
        metrics:
            - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metric value must be a map for index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
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
        metrics:
            - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label missing for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label:
                - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label:
                - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
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
        metrics:
            - label: test
              description:
                - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "description must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label: test
              description:
                - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "description must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix:
                - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_prefix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix:
                - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_prefix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix:
                - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_suffix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix:
                - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_suffix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "value missing for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value:
                - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "value must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value:
                - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "value must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value: test
              precision:
                - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "precision must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
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
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value: test
              precision:
                - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "precision must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
    }

    #[test]
    fn test_deployments() {
        let yaml = r#"
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
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field deployments".to_string())
        );

        let yaml = r#"
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
        test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("scenario missing for deployment \"deployment1\"".to_string())
        );

        let yaml = r#"
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
        test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("order missing for deployment \"deployment1\"".to_string())
        );
    }

    #[test]
    fn test_gui() {
        let yaml = r#"
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
gui:
    test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name missing for gui".to_string())
        );
        let yaml = r#"
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
gui:
    name:
      - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name must be a string".to_string())
        );
        let yaml = r#"
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
gui:
    name:
      - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name must be a string".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("description missing for gui".to_string())
        );
        let yaml = r#"
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
gui:
    name: test
    description:
      - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("description must be a string".to_string())
        );
        let yaml = r#"
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
gui:
    name: test
    description:
      - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("description must be a string".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deployments missing for gui".to_string())
        );
        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deployments must be a vector".to_string())
        );
        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deployments must be a vector".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deployment missing for gui deployment index 0".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name missing for gui deployment index 0".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("description missing for gui deployment index 0".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deposits missing for gui deployment index 0".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token missing for deposit index 0 in gui deployment index 0".to_string()
            )
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "presets missing for deposit index 0 in gui deployment index 0".to_string()
            )
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "preset value must be a string for preset index 0 in deposit index 0 in gui deployment index 0".to_string()
            )
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("fields missing for gui deployment index 0".to_string())
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "binding missing for field index 0 in gui deployment index 0".to_string()
            )
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - binding: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "name missing for field index 0 in gui deployment index 0".to_string()
            )
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - binding: test
              name: test
              presets:
                - value:
                    - test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "preset value must be a string for preset index 0 in field index 0 in gui deployment index 0".to_string()
            )
        );

        let yaml = r#"
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
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - binding: test
              name: test
              presets:
                - value: test
          select-tokens:
            - test: test
"#;
        let error = DotrainYaml::new(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "select-token value must be a string for select-token index 0 in gui deployment index 0".to_string()
            )
        );
    }
}
