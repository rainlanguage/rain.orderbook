use std::{collections::HashMap, sync::Arc};

use thiserror::Error;

use crate::*;

#[derive(Debug)]
pub struct Chart {
    pub scenario: Arc<Scenario>,
    pub plots: HashMap<String, Plot>,
}

#[derive(Debug)]
pub struct Plot {
    pub data: DataPoints,
    pub plot_type: String,
}

#[derive(Debug)]
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
