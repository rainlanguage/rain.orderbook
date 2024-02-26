use crate::*;
use std::{collections::HashMap, num::ParseIntError, sync::Arc};
use thiserror::Error;

#[derive(Debug)]
pub struct Scenario {
    pub bindings: HashMap<String, String>,
    pub runs: Option<u64>,
    pub network: Option<Arc<Network>>,
    pub deployer: Option<Arc<Deployer>>,
    pub orderbook: Option<Arc<Orderbook>>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseScenarioStringError {
    #[error("Failed to parse runs")]
    RunsParseError(ParseIntError),
    #[error("Parent binding shadowed by child: {0}")]
    ParentBindingShadowedError(String),
    #[error("Parent network shadowed by child: {0}")]
    ParentNetworkShadowedError(String),
    #[error("Parent deployer shadowed by child: {0}")]
    ParentDeployerShadowedError(String),
    #[error("Parent orderbook shadowed by child: {0}")]
    ParentOrderbookShadowedError(String),
}

// Shadowing is disallowed for networks, deployers, orderbooks and specific bindings.
// If a child specifies one that is already set by the parent, this is an error.
//
// Nested scenarios within the ScenarioString struct are flattened out into a
// hashmap of scenarios, where the key is the path such as foo.bar.baz.
// Every level of the scenario path inherits its parents bindings recursively.
impl ScenarioString {
    pub fn try_into_scenarios(
        &self,
        name: String,
        parent_bindings: Option<&HashMap<String, String>>,
        parent_network: Option<&Arc<Network>>,
        parent_deployer: Option<&Arc<Deployer>>,
        parent_orderbook: Option<&Arc<Orderbook>>,
        networks: &HashMap<String, Arc<Network>>,
        deployers: &HashMap<String, Arc<Deployer>>,
        orderbooks: &HashMap<String, Arc<Orderbook>>,
    ) -> Result<HashMap<String, Arc<Scenario>>, ParseScenarioStringError> {
        // Handling Network
        let network_ref = self
            .network
            .as_ref()
            .map(|network_name| {
                networks.get(network_name).ok_or_else(|| {
                    ParseScenarioStringError::ParentNetworkShadowedError(name.clone())
                })
            })
            .transpose()?
            .or_else(|| parent_network);

        // Handling Deployer
        let deployer_ref = self
            .deployer
            .as_ref()
            .map(|deployer_name| {
                deployers.get(deployer_name).ok_or_else(|| {
                    ParseScenarioStringError::ParentDeployerShadowedError(name.clone())
                })
            })
            .transpose()?
            .or_else(|| parent_deployer);

        // Handling Orderbook
        let orderbook_ref = self
            .orderbook
            .as_ref()
            .map(|orderbook_name| {
                orderbooks.get(orderbook_name).ok_or_else(|| {
                    ParseScenarioStringError::ParentOrderbookShadowedError(name.clone())
                })
            })
            .transpose()?
            .or_else(|| parent_orderbook);

        // Merge bindings and check for shadowing
        let mut bindings = parent_bindings.map_or_else(HashMap::new, |pb| pb.clone());
        for (k, v) in &self.bindings {
            if let Some(parent_value) = parent_bindings.and_then(|pb| pb.get(k)) {
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
            network: network_ref.cloned(),
            deployer: deployer_ref.cloned(),
            orderbook: orderbook_ref.cloned(),
        });

        let mut scenarios = HashMap::new();
        scenarios.insert(name.clone(), parent_scenario);

        // Recursively add child scenarios
        if let Some(scenarios_map) = &self.scenarios {
            for (child_name, child_scenario) in scenarios_map {
                let child_scenarios = child_scenario.try_into_scenarios(
                    format!("{}.{}", name, child_name),
                    Some(&bindings),
                    network_ref,
                    deployer_ref,
                    orderbook_ref,
                    networks,
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
        let parent_bindings = {
            let mut bindings = HashMap::new();
            bindings.insert("shared_key".to_string(), "parent_value".to_string());
            bindings
        };

        let mut child_bindings = HashMap::new();
        child_bindings.insert("shared_key".to_string(), "child_value".to_string()); // Intentionally shadowing parent binding

        let child_scenario = ScenarioString {
            bindings: child_bindings,
            runs: None,
            network: None,
            deployer: None,
            orderbook: None,
            scenarios: None,
        };

        let result = child_scenario.try_into_scenarios(
            "child".to_string(),
            Some(&parent_bindings),
            None,            // Assuming no parent network for simplification
            None,            // Assuming no parent deployer for simplification
            None,            // Assuming no parent orderbook for simplification
            &HashMap::new(), // Empty networks for simplification
            &HashMap::new(), // Empty deployers for simplification
            &HashMap::new(), // Empty orderbooks for simplification
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
            network: Some("mainnet".to_string()),
            deployer: None,
            orderbook: None,
            scenarios: None,
        };

        let child_scenario_bindings = HashMap::new();

        let child_scenario = ScenarioString {
            bindings: child_scenario_bindings,
            runs: Some("5".to_string()),
            network: None, // Intentionally left None to test inheritance
            deployer: None,
            orderbook: None,
            scenarios: None,
        };

        let mut networks = HashMap::new();
        networks.insert(
            "mainnet".to_string(),
            Arc::new(Network {
                rpc: "https://mainnet.node".parse().unwrap(),
                chain_id: 1,
                label: Some("Ethereum Mainnet".to_string()),
                network_id: Some(1),
                currency: Some("ETH".to_string()),
            }),
        );

        // Convert root scenario
        let root_result = root_scenario.try_into_scenarios(
            "root".to_string(),
            None,
            None,
            None,
            None,
            &networks,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(root_result.is_ok());
        let root_scenarios = root_result.unwrap();
        let root_converted = root_scenarios.get("root").unwrap();

        // Convert child scenario with the root's network context
        let child_result = child_scenario.try_into_scenarios(
            "child".to_string(),
            Some(&root_converted.bindings),
            Some(&root_converted.network.as_ref().unwrap()),
            None,
            None,
            &networks,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(child_result.is_ok());
        let child_scenarios = child_result.unwrap();
        let child_converted = child_scenarios.get("child").unwrap();

        // Verify child inherited network from root
        assert_eq!(child_converted.network, root_converted.network);
    }
}
