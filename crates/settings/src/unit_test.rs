use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct UnitTestConfigSource {
    pub test: TestConfigSource,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(UnitTestConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TestConfigSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculate_entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle_entrypoint: Option<String>,
    pub scenario_name: String,
    pub scenario: ScenarioConfigSource,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TestConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TestConfig {
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub calculate_entrypoint: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub handle_entrypoint: Option<String>,
    pub scenario_name: String,
    pub scenario: Arc<ScenarioCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TestConfig);

impl TestConfigSource {
    pub fn try_into_test_config(self) -> Result<TestConfig, ParseConfigSourceError> {
        let mut bindings = HashMap::new();
        for (k, v) in &self.scenario.bindings {
            bindings.insert(k.to_string(), v.to_string());
        }

        let scenario = Arc::new(ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: self.scenario_name.clone(),
            bindings: bindings.clone(),
            runs: self.scenario.runs,
            blocks: self.scenario.blocks.clone(),
            deployer: Arc::new(DeployerCfg::dummy()),
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
