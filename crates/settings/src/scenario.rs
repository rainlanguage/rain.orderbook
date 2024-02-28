use crate::*;
use std::{collections::HashMap, num::ParseIntError, sync::Arc};
use thiserror::Error;

#[derive(Debug)]
pub struct Scenario {
    pub bindings: HashMap<String, String>,
    pub runs: Option<u64>,
    pub deployer: Arc<Deployer>,
    pub orderbook: Option<Arc<Orderbook>>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseScenarioStringError {
    #[error("Failed to parse runs")]
    RunsParseError(ParseIntError),
    #[error("Parent binding shadowed by child: {0}")]
    ParentBindingShadowedError(String),
    #[error("Parent deployer shadowed by child: {0}")]
    ParentDeployerShadowedError(String),
    #[error("Deployer not found: {0}")]
    DeployerNotFound(String),
    #[error("Parent orderbook shadowed by child: {0}")]
    ParentOrderbookShadowedError(String),
    #[error("Orderbook not found: {0}")]
    OrderbookNotFound(String),
}

#[derive(Default)]
pub struct ScenarioParent {
    bindings: Option<HashMap<String, String>>,
    deployer: Option<Arc<Deployer>>,
    orderbook: Option<Arc<Orderbook>>,
}

// Shadowing is disallowed for deployers, orderbooks and specific bindings.
// If a child specifies one that is already set by the parent, this is an error.
//
// Nested scenarios within the ScenarioString struct are flattened out into a
// hashmap of scenarios, where the key is the path such as foo.bar.baz.
// Every level of the scenario path inherits its parents bindings recursively.
impl ScenarioString {
    pub fn try_into_scenarios(
        &self,
        name: String,
        parent: &ScenarioParent,
        deployers: &HashMap<String, Arc<Deployer>>,
        orderbooks: &HashMap<String, Arc<Orderbook>>,
    ) -> Result<HashMap<String, Arc<Scenario>>, ParseScenarioStringError> {
        // Try to resolve the deployer by the specified name or fall back to the scenario name
        let resolved_name = self.deployer.as_ref().unwrap_or(&name);
        let deployer_ref = deployers
            .get(resolved_name)
            .ok_or_else(|| ParseScenarioStringError::DeployerNotFound(resolved_name.clone()))?;

        // Perform shadowing check if there's a parent
        // Check if the parent scenario has a deployer and if it's different from the resolved deployer
        if let Some(parent_deployer_name) = &parent.deployer {
            if parent_deployer_name.label != Some(resolved_name.to_string()) {
                return Err(ParseScenarioStringError::ParentDeployerShadowedError(
                    resolved_name.clone(),
                ));
            }
        }

        // Try to resolve the orderbook by the specified name or fall back to the scenario name
        let resolved_name = self.orderbook.as_ref().unwrap_or(&name);
        let orderbook_ref = orderbooks.get(resolved_name);

        // Perform shadowing check if we resolved an orderbook and there's also a parent
        if let Some(parent_orderbook_name) = &parent.orderbook {
            if parent_orderbook_name.label != Some(resolved_name.to_string()) {
                return Err(ParseScenarioStringError::ParentOrderbookShadowedError(
                    resolved_name.clone(),
                ));
            }
        }

        // Merge bindings and check for shadowing
        let mut bindings = parent
            .bindings
            .as_ref()
            .map_or_else(HashMap::new, |pb| pb.clone());
        for (k, v) in &self.bindings {
            if let Some(parent_value) = parent.bindings.as_ref().and_then(|pb| pb.get(k)) {
                if parent_value != v {
                    return Err(ParseScenarioStringError::ParentBindingShadowedError(
                        k.to_string(),
                    ));
                }
            }
            bindings.insert(k.to_string(), v.to_string());
        }

        // Create and add the parent scenario for this level
        let parent_scenario = Arc::new(Scenario {
            bindings: bindings.clone(),
            runs: self
                .runs
                .as_ref()
                .map(|s| s.parse::<u64>())
                .transpose()
                .map_err(ParseScenarioStringError::RunsParseError)?,
            deployer: deployer_ref.clone(),
            orderbook: orderbook_ref.cloned(),
        });

        let mut scenarios = HashMap::new();
        scenarios.insert(name.clone(), parent_scenario);

        // Recursively add child scenarios
        if let Some(scenarios_map) = &self.scenarios {
            for (child_name, child_scenario) in scenarios_map {
                let child_scenarios = child_scenario.try_into_scenarios(
                    format!("{}.{}", name, child_name),
                    &ScenarioParent {
                        bindings: Some(bindings.clone()),
                        deployer: Some(deployer_ref.clone()),
                        orderbook: orderbook_ref.cloned(),
                    },
                    deployers,
                    orderbooks,
                )?;

                scenarios.extend(child_scenarios);
            }
        }

        Ok(scenarios)
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_scenario_shadowing_error_in_bindings() {
        let parent_bindings =
            HashMap::from([("shared_key".to_string(), "parent_value".to_string())]);

        let parent_scenario = ScenarioParent {
            bindings: Some(parent_bindings),
            deployer: None,
            orderbook: None,
        };

        let mut child_bindings = HashMap::new();
        child_bindings.insert("shared_key".to_string(), "child_value".to_string()); // Intentionally shadowing parent binding

        let child_scenario = ScenarioString {
            bindings: child_bindings,
            runs: None,
            deployer: None,
            orderbook: None,
            scenarios: None,
        };

        let result = child_scenario.try_into_scenarios(
            "child".to_string(),
            &parent_scenario, // Assuming no parent orderbook for simplification
            &HashMap::new(),  // Empty deployers for simplification
            &HashMap::new(),  // Empty orderbooks for simplification
        );

        assert!(result.is_err());
        match result.err().unwrap() {
            ParseScenarioStringError::ParentBindingShadowedError(key) => {
                assert_eq!(key, "shared_key");
            }
            _ => panic!("Expected ParentBindingShadowedError"),
        }
    }

    #[test]
    fn test_scenario_network_inheritance() {
        let root_scenario_bindings = HashMap::new();

        let root_scenario = ScenarioString {
            bindings: root_scenario_bindings,
            runs: Some("10".to_string()),
            deployer: None,
            orderbook: None,
            scenarios: None,
        };

        let child_scenario_bindings = HashMap::new();

        let child_scenario = ScenarioString {
            bindings: child_scenario_bindings,
            runs: Some("5".to_string()),
            deployer: None,
            orderbook: None,
            scenarios: None,
        };

        // Convert root scenario
        let root_result = root_scenario.try_into_scenarios(
            "root".to_string(),
            &ScenarioParent::default(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(root_result.is_ok());
        let root_scenarios = root_result.unwrap();
        let root_converted = root_scenarios.get("root").unwrap();

        // Convert child scenario with the root's network context
        let child_result = child_scenario.try_into_scenarios(
            "child".to_string(),
            &ScenarioParent {
                bindings: Some(root_converted.bindings.clone()),
                deployer: None,
                orderbook: None,
            },
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(child_result.is_ok());
        let child_scenarios = child_result.unwrap();
        let child_converted = child_scenarios.get("child").unwrap();
    }
}
