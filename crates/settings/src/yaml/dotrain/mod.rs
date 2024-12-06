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
