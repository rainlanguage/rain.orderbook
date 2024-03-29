use crate::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Chart {
    #[typeshare(typescript(type = "Scenario"))]
    pub scenario: Arc<Scenario>,
    pub plots: Vec<Plot>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseChartConfigSourceError {
    #[error("Scenario not found: {0}")]
    ScenarioNotFoundError(String),
}

impl ChartConfigSource {
    pub fn try_into_chart(
        self,
        name: String,
        scenarios: &HashMap<String, Arc<Scenario>>,
    ) -> Result<Chart, ParseChartConfigSourceError> {
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

        Ok(Chart {
            scenario: scenario_ref,
            plots: self
                .plots
                .into_iter()
                .map(|(name, plot)| {
                    // if the plot has a title, use it, otherwise use the name
                    let title = plot.title.unwrap_or_else(|| name.clone());
                    Plot {
                        title: Some(title),
                        ..plot
                    }
                })
                .collect::<Vec<Plot>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test::mock_plot;

    use self::test::mock_deployer;

    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_scenario(name: &str, runs: Option<u64>) -> (String, Arc<Scenario>) {
        let scenario = Scenario {
            name: name.into(),
            bindings: HashMap::from([(String::from("key"), String::from("value"))]), // Example binding
            runs,
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
            plots,
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
            plots,
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
        let scenarios = HashMap::<String, Arc<Scenario>>::new(); // No scenarios added

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartConfigSource {
            scenario: Some("nonexistent_scenario".to_string()),
            plots,
        };

        let result = chart_string.try_into_chart("chart3".to_string(), &scenarios);
        assert!(matches!(
            result,
            Err(ParseChartConfigSourceError::ScenarioNotFoundError(_))
        ));
    }

    #[test]
    fn test_no_scenario_matching_chart_name() {
        let scenarios = HashMap::<String, Arc<Scenario>>::new(); // No scenarios added

        let chart_string = ChartConfigSource {
            scenario: None,
            plots: HashMap::new(),
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

        let chart_string = ChartConfigSource {
            scenario: Some(scenario_name),
            plots,
        };

        let chart = chart_string
            .try_into_chart("chart5".to_string(), &scenarios)
            .unwrap();
        assert!(Arc::ptr_eq(
            &chart.scenario,
            scenarios.get("scenario5").unwrap()
        ));
        assert_eq!(chart.plots.len(), 2);

        // both plots should have the name "Title"
        let mut plots = chart
            .plots
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
