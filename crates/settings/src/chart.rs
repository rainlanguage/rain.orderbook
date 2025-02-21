use crate::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ChartCfg {
    pub scenario: Arc<ScenarioCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub plots: Option<Vec<PlotCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub metrics: Option<Vec<MetricCfg>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ChartCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct MetricCfg {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_suffix: Option<String>,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<u8>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(MetricCfg);

#[derive(Error, Debug, PartialEq)]
pub enum ParseChartConfigSourceError {
    #[error("Scenario not found: {0}")]
    ScenarioNotFoundError(String),
}

impl ChartConfigSource {
    pub fn try_into_chart(
        self,
        name: String,
        scenarios: &HashMap<String, Arc<ScenarioCfg>>,
    ) -> Result<ChartCfg, ParseChartConfigSourceError> {
        let scenario_ref = match self.scenario {
            Some(scenario_name) => scenarios
                .get(&scenario_name)
                .ok_or(ParseChartConfigSourceError::ScenarioNotFoundError(
                    scenario_name.clone(),
                ))
                .map(Arc::clone)?,
            None => scenarios
                .get(&name)
                .ok_or(ParseChartConfigSourceError::ScenarioNotFoundError(
                    name.clone(),
                ))
                .map(Arc::clone)?,
        };

        // Convert `self.plots` from Option<HashMap<String, Plot>> to Option<Vec<Plot>>
        let plots = self.plots.map(|plots_map| {
            plots_map
                .into_iter()
                .map(|(name, mut plot)| {
                    // If the plot does not have a title, use the name from the map
                    plot.title.get_or_insert(name);
                    plot
                })
                .collect::<Vec<PlotCfg>>()
        });

        Ok(ChartCfg {
            scenario: scenario_ref,
            metrics: self.metrics,
            plots,
        })
    }
}

#[cfg(test)]
mod tests {
    use strict_yaml_rust::StrictYaml;

    use crate::test::mock_plot;

    use self::test::mock_deployer;

    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    fn create_scenario(name: &str, runs: Option<u64>) -> (String, Arc<ScenarioCfg>) {
        let scenario = ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: name.into(),
            bindings: HashMap::from([(String::from("key"), String::from("value"))]), // Example binding
            runs,
            blocks: None,
            deployer: mock_deployer(),
        };
        (name.to_string(), Arc::new(scenario))
    }

    #[test]
    fn test_success_explicit_scenario_name() {
        let (scenario_name, scenario) = create_scenario("scenario1", 100.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(scenario_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartConfigSource {
            scenario: Some(scenario_name),
            plots: Some(plots),
            metrics: None,
        };

        let chart = chart_string
            .try_into_chart("chart1".to_string(), &scenarios)
            .unwrap();
        assert!(Arc::ptr_eq(
            &chart.scenario,
            scenarios.get("scenario1").unwrap()
        ));
    }

    #[test]
    fn test_success_using_chart_name() {
        let (chart_name, scenario) = create_scenario("chart2", 100.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(chart_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartConfigSource {
            scenario: None,
            plots: Some(plots),
            metrics: None,
        };

        let chart = chart_string
            .try_into_chart(chart_name.clone(), &scenarios)
            .unwrap();
        assert!(Arc::ptr_eq(
            &chart.scenario,
            scenarios.get(&chart_name).unwrap()
        ));
    }

    #[test]
    fn test_scenario_not_found_error() {
        let scenarios = HashMap::<String, Arc<ScenarioCfg>>::new(); // No scenarios added

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartConfigSource {
            scenario: Some("nonexistent_scenario".to_string()),
            plots: Some(plots),
            metrics: None,
        };

        let result = chart_string.try_into_chart("chart3".to_string(), &scenarios);
        assert!(matches!(
            result,
            Err(ParseChartConfigSourceError::ScenarioNotFoundError(_))
        ));
    }

    #[test]
    fn test_no_scenario_matching_chart_name() {
        let scenarios = HashMap::<String, Arc<ScenarioCfg>>::new(); // No scenarios added

        let chart_string = ChartConfigSource {
            scenario: None,
            plots: None,
            metrics: None,
        };

        let result = chart_string.try_into_chart("chart4".to_string(), &scenarios);
        assert!(matches!(
            result,
            Err(ParseChartConfigSourceError::ScenarioNotFoundError(_))
        ));
    }

    #[test]
    fn test_multiple_plots() {
        let (scenario_name, scenario) = create_scenario("scenario5", 200.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(scenario_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let (plot_name, plot) = mock_plot("plot2");
        plots.insert(plot_name, plot);

        let metrics: Vec<MetricCfg> = vec![MetricCfg {
            label: "label".to_string(),
            description: Some("description".to_string()),
            unit_prefix: Some("unit_prefix".to_string()),
            unit_suffix: Some("unit_suffix".to_string()),
            value: "value".to_string(),
            precision: Some(2),
        }];

        let chart_string = ChartConfigSource {
            scenario: Some(scenario_name),
            plots: Some(plots),
            metrics: Some(metrics),
        };

        let chart = chart_string
            .try_into_chart("chart5".to_string(), &scenarios)
            .unwrap();
        assert!(Arc::ptr_eq(
            &chart.scenario,
            scenarios.get("scenario5").unwrap()
        ));
        assert_eq!(chart.clone().plots.unwrap().len(), 2);

        // both plots should have the name "Title"
        let mut plots = chart
            .plots
            .unwrap()
            .iter()
            .map(|p| p.title.clone())
            .collect::<Vec<Option<String>>>();
        plots.sort();
        assert_eq!(
            plots,
            vec![Some("Title".to_string()), Some("Title".to_string())]
        );
    }
}
