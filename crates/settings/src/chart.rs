use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

use crate::*;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chart {
    #[typeshare(typescript(type = "Scenario"))]
    pub scenario: Arc<Scenario>,
    pub plots: HashMap<String, Plot>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plot {
    pub data: DataPoints,
    pub plot_type: String,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataPoints {
    pub x: String,
    pub y: String,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseChartStringError {
    #[error("Scenario not found: {0}")]
    ScenarioNotFoundError(String),
}

impl ChartString {
    pub fn try_into_chart(
        self,
        name: String,
        scenarios: &HashMap<String, Arc<Scenario>>,
    ) -> Result<Arc<Chart>, ParseChartStringError> {
        let scenario_ref = match self.scenario {
            Some(scenario_name) => scenarios
                .get(&scenario_name)
                .ok_or(ParseChartStringError::ScenarioNotFoundError(
                    scenario_name.clone(),
                ))
                .map(Arc::clone)?,
            None => scenarios
                .get(&name)
                .ok_or(ParseChartStringError::ScenarioNotFoundError(name.clone()))
                .map(Arc::clone)?,
        };

        Ok(Arc::new(Chart {
            scenario: scenario_ref,
            plots: self
                .plots
                .into_iter()
                .map(|(name, plot)| {
                    Ok((
                        name,
                        Plot {
                            data: DataPoints {
                                x: plot.data.x,
                                y: plot.data.y,
                            },
                            plot_type: plot.plot_type,
                        },
                    ))
                })
                .collect::<Result<HashMap<String, Plot>, ParseChartStringError>>()?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use self::test::mock_deployer;

    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_scenario(name: &str, runs: Option<u64>) -> (String, Arc<Scenario>) {
        let scenario = Scenario {
            bindings: HashMap::from([(String::from("key"), String::from("value"))]), // Example binding
            runs,
            deployer: mock_deployer(),
            orderbook: None,
        };
        (name.to_string(), Arc::new(scenario))
    }

    fn create_plot(name: &str) -> (String, PlotString) {
        (
            name.to_string(),
            PlotString {
                plot_type: "line".to_string(),
                data: DataPointsString {
                    x: "time".to_string(),
                    y: "value".to_string(),
                },
            },
        )
    }

    #[test]
    fn test_success_explicit_scenario_name() {
        let (scenario_name, scenario) = create_scenario("scenario1", 100.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(scenario_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = create_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartString {
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
        assert!(chart.plots.contains_key("plot1"));
    }

    #[test]
    fn test_success_using_chart_name() {
        let (chart_name, scenario) = create_scenario("chart2", 100.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(chart_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = create_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartString {
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
        assert!(chart.plots.contains_key("plot1"));
    }

    #[test]
    fn test_scenario_not_found_error() {
        let scenarios = HashMap::<String, Arc<Scenario>>::new(); // No scenarios added

        let mut plots = HashMap::new();
        let (plot_name, plot) = create_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartString {
            scenario: Some("nonexistent_scenario".to_string()),
            plots,
        };

        let result = chart_string.try_into_chart("chart3".to_string(), &scenarios);
        assert!(matches!(
            result,
            Err(ParseChartStringError::ScenarioNotFoundError(_))
        ));
    }

    #[test]
    fn test_no_scenario_matching_chart_name() {
        let scenarios = HashMap::<String, Arc<Scenario>>::new(); // No scenarios added

        let chart_string = ChartString {
            scenario: None,
            plots: HashMap::new(),
        };

        let result = chart_string.try_into_chart("chart4".to_string(), &scenarios);
        assert!(matches!(
            result,
            Err(ParseChartStringError::ScenarioNotFoundError(_))
        ));
    }

    #[test]
    fn test_multiple_plots() {
        let (scenario_name, scenario) = create_scenario("scenario5", 200.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(scenario_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = create_plot("plot1");
        plots.insert(plot_name, plot);

        let (plot_name, plot) = create_plot("plot2");
        plots.insert(plot_name, plot);

        let chart_string = ChartString {
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
        assert!(chart.plots.contains_key("plot1") && chart.plots.contains_key("plot2"));
    }
}
