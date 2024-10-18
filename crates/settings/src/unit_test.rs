use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct UnitTestConfigSource {
    pub test: TestConfigSource,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TestConfigSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculate_entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle_entrypoint: Option<String>,
    pub scenario_name: String,
    pub scenario: ScenarioConfigSource,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TestConfig {
    pub calculate_entrypoint: Option<String>,
    pub handle_entrypoint: Option<String>,
    pub scenario_name: String,
    #[typeshare(typescript(type = "Scenario"))]
    pub scenario: Arc<Scenario>,
}

impl TestConfigSource {
    pub fn try_into_test_config(self) -> Result<TestConfig, ParseConfigSourceError> {
        let mut bindings = HashMap::new();
        for (k, v) in &self.scenario.bindings {
            bindings.insert(k.to_string(), v.to_string());
        }

        let scenario = Arc::new(Scenario {
            name: self.scenario_name.clone(),
            bindings: bindings.clone(),
            runs: self.scenario.runs,
            blocks: self.scenario.blocks.clone(),
            deployer: Arc::new(Deployer::dummy()),
        });

        let config = TestConfig {
            calculate_entrypoint: self.calculate_entrypoint,
            handle_entrypoint: self.handle_entrypoint,
            scenario_name: self.scenario_name.clone(),
            scenario,
        };

        Ok(config)
    }
}
