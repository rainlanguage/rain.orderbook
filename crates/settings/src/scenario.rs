use crate::*;
use blocks::Blocks;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, num::ParseIntError, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct Scenario {
    pub name: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "Record<string, string>"))]
    pub bindings: HashMap<String, String>,
    #[typeshare(typescript(type = "number"))]
    pub runs: Option<u64>,
    #[typeshare(skip)]
    pub blocks: Option<Blocks>,
    pub entrypoint: Option<String>,
    #[typeshare(typescript(type = "Deployer"))]
    pub deployer: Arc<Deployer>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Scenario);

#[derive(Error, Debug, PartialEq)]
pub enum ParseScenarioConfigSourceError {
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
}

#[derive(Default)]
pub struct ScenarioParent {
    bindings: Option<HashMap<String, String>>,
    deployer: Option<Arc<Deployer>>,
}

// Shadowing is disallowed for deployers, orderbooks and specific bindings.
// If a child specifies one that is already set by the parent, this is an error.
//
// Nested scenarios within the ScenarioConfigSource struct are flattened out into a
// hashmap of scenarios, where the key is the path such as foo.bar.baz.
// Every level of the scenario path inherits its parents bindings recursively.
impl ScenarioConfigSource {
    pub fn try_into_scenarios(
        &self,
        name: String,
        parent: &ScenarioParent,
        deployers: &HashMap<String, Arc<Deployer>>,
    ) -> Result<HashMap<String, Arc<Scenario>>, ParseScenarioConfigSourceError> {
        // Determine the resolved name for the deployer, preferring the explicit deployer name if provided.
        let resolved_name = self.deployer.as_ref().unwrap_or(&name);

        // Attempt to find the deployer using the resolved name.
        let resolved_deployer = deployers.get(resolved_name);

        // If no deployer is found using the resolved name, fall back to the parent's deployer, if any.
        let deployer_ref = resolved_deployer.or(parent.deployer.as_ref());

        // If no deployer could be resolved and there's no parent deployer, return an error.
        let deployer_ref = deployer_ref
            .ok_or_else(|| ParseScenarioConfigSourceError::DeployerNotFound(name.clone()))?;

        // Check for non-matching override: if both the current and parent deployers are present and different, it's an error.
        if let (deployer, Some(parent_deployer)) = (deployer_ref, parent.deployer.as_ref()) {
            if deployer.label != parent_deployer.label {
                return Err(ParseScenarioConfigSourceError::ParentDeployerShadowedError(
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
                    return Err(ParseScenarioConfigSourceError::ParentBindingShadowedError(
                        k.to_string(),
                    ));
                }
            }
            bindings.insert(k.to_string(), v.to_string());
        }

        // Create and add the parent scenario for this level
        let parent_scenario = Arc::new(Scenario {
            name: name.clone(),
            bindings: bindings.clone(),
            runs: self.runs,
            blocks: self.blocks.clone(),
            entrypoint: self.entrypoint.clone(),
            deployer: deployer_ref.clone(),
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
                    },
                    deployers,
                )?;

                scenarios.extend(child_scenarios);
            }
        }

        Ok(scenarios)
    }
}

#[cfg(test)]

mod tests {
    use crate::test::mock_deployer;
    use alloy::primitives::Address;
    use url::Url;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_scenarios_conversion_with_nesting() {
        // Initialize networks as in the previous example
        let mut networks = HashMap::new();
        networks.insert(
            "mainnet".to_string(),
            NetworkConfigSource {
                rpc: Url::parse("https://mainnet.node").unwrap(),
                chain_id: 1,
                label: Some("Ethereum Mainnet".to_string()),
                network_id: Some(1),
                currency: Some("ETH".to_string()),
            },
        );

        // Define a deployer
        let mut deployers = HashMap::new();
        deployers.insert(
            "mainnet".to_string(),
            DeployerConfigSource {
                address: "0xabcdef0123456789ABCDEF0123456789ABCDEF01"
                    .parse::<Address>()
                    .unwrap(),
                network: None,
                label: Some("Mainnet Deployer".to_string()),
            },
        );

        // Define nested scenarios
        let mut nested_scenario2 = HashMap::new();
        nested_scenario2.insert(
            "nested_scenario2".to_string(),
            ScenarioConfigSource {
                bindings: HashMap::new(), // Assuming no bindings for simplification
                runs: Some(2),
                blocks: None,
                entrypoint: None,
                deployer: None,
                scenarios: None, // No further nesting
            },
        );

        let mut nested_scenario1 = HashMap::new();
        nested_scenario1.insert(
            "nested_scenario1".to_string(),
            ScenarioConfigSource {
                bindings: HashMap::new(), // Assuming no bindings for simplification
                runs: Some(5),
                blocks: None,
                entrypoint: None,
                deployer: None,
                scenarios: Some(nested_scenario2), // Include nested_scenario2
            },
        );

        // Define root scenario with nested_scenario1
        let mut scenarios = HashMap::new();
        scenarios.insert(
            "root_scenario".to_string(),
            ScenarioConfigSource {
                bindings: HashMap::new(), // Assuming no bindings for simplification
                runs: Some(10),
                blocks: None,
                entrypoint: None,
                deployer: Some("mainnet".to_string()),
                scenarios: Some(nested_scenario1), // Include nested_scenario1
            },
        );

        // Construct ConfigSource with the above scenarios
        let config_string = ConfigSource {
            raindex_version: None,
            using_networks_from: HashMap::new(),
            networks,
            subgraphs: HashMap::new(), // Assuming no subgraphs for simplification
            metaboards: HashMap::new(), // Assuming no metaboards for simplification
            orderbooks: HashMap::new(), // Assuming no orderbooks for simplification
            tokens: HashMap::new(),    // Assuming no tokens for simplification
            deployers,
            orders: HashMap::new(), // Assuming no orders for simplification
            scenarios,
            charts: HashMap::new(), // Assuming no charts for simplification
            deployments: HashMap::new(),
            sentry: None,
            accounts: None, // Assuming no accounts for simplification
            gui: None,
        };

        // Perform the conversion
        let config_result = Config::try_from(config_string);
        println!("{:?}", config_result);
        assert!(config_result.is_ok());

        let config = config_result.unwrap();

        // Verify the root scenario
        assert!(config.scenarios.contains_key("root_scenario"));
        let root_scenario = config.scenarios.get("root_scenario").unwrap();
        assert_eq!(root_scenario.runs, Some(10));

        // Verify the first level of nested scenarios
        assert!(config
            .scenarios
            .contains_key("root_scenario.nested_scenario1"));
        let nested_scenario1 = config
            .scenarios
            .get("root_scenario.nested_scenario1")
            .unwrap();
        assert_eq!(nested_scenario1.runs, Some(5));

        // Verify the second level of nested scenarios
        assert!(config
            .scenarios
            .contains_key("root_scenario.nested_scenario1.nested_scenario2"));
        let nested_scenario2 = config
            .scenarios
            .get("root_scenario.nested_scenario1.nested_scenario2")
            .unwrap();
        assert_eq!(nested_scenario2.runs, Some(2));
    }

    #[test]
    fn test_scenario_shadowing_error_in_bindings() {
        let parent_bindings =
            HashMap::from([("shared_key".to_string(), "parent_value".to_string())]);

        let parent_scenario = ScenarioParent {
            bindings: Some(parent_bindings),
            deployer: Some(mock_deployer()),
        };

        let mut child_bindings = HashMap::new();
        child_bindings.insert("shared_key".to_string(), "child_value".to_string()); // Intentionally shadowing parent binding

        let child_scenario = ScenarioConfigSource {
            bindings: child_bindings,
            runs: None,
            blocks: None,
            entrypoint: None,
            deployer: None,
            scenarios: None,
        };

        let result = child_scenario.try_into_scenarios(
            "child".to_string(),
            &parent_scenario,
            &HashMap::new(), // Empty deployers for simplification
        );

        println!("{:?}", result);

        assert!(result.is_err());
        match result.err().unwrap() {
            ParseScenarioConfigSourceError::ParentBindingShadowedError(key) => {
                assert_eq!(key, "shared_key");
            }
            _ => panic!("Expected ParentBindingShadowedError"),
        }
    }
}
