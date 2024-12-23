use crate::*;
use blocks::Blocks;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::ParseIntError,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
use typeshare::typeshare;
use yaml::{
    default_document, optional_hash, optional_string, require_hash, require_string, YamlError,
    YamlParsableHash,
};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct Scenario {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "Record<string, string>"))]
    pub bindings: HashMap<String, String>,
    #[typeshare(typescript(type = "number"))]
    pub runs: Option<u64>,
    #[typeshare(skip)]
    pub blocks: Option<Blocks>,
    #[typeshare(typescript(type = "Deployer"))]
    pub deployer: Arc<Deployer>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Scenario);

impl Scenario {
    pub fn validate_runs(value: &str) -> Result<u64, ParseScenarioConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseScenarioConfigSourceError::RunsParseError)
    }

    pub fn validate_blocks(value: &str) -> Result<Blocks, ParseScenarioConfigSourceError> {
        match serde_yaml::from_str::<Blocks>(value) {
            Ok(blocks) => Ok(blocks),
            Err(_) => Err(ParseScenarioConfigSourceError::BlocksParseError(
                value.to_string(),
            )),
        }
    }

    pub fn validate_scenario(
        document: Arc<RwLock<StrictYaml>>,
        deployer: &mut Option<Arc<Deployer>>,
        scenarios: &mut HashMap<String, Scenario>,
        parent_scenario: ScenarioParent,
        scenario_key: String,
        scenario_yaml: &StrictYaml,
    ) -> Result<(), YamlError> {
        let current_bindings = require_hash(
            scenario_yaml,
            Some("bindings"),
            Some(format!("bindings map missing in scenario: {scenario_key}")),
        )?
        .iter()
        .map(|(binding_key, binding_value)| {
            let binding_key = binding_key.as_str().unwrap_or_default();
            Ok((
                binding_key.to_string(),
                require_string(
                    binding_value,
                    None,
                    Some(format!(
                        "binding value must be a string for key: {binding_key} in scenario: {scenario_key}",
                    )),
                )?,
            ))
        })
        .collect::<Result<HashMap<_, _>, YamlError>>()?;

        let mut bindings = parent_scenario
            .bindings
            .as_ref()
            .map_or_else(HashMap::new, |pb| pb.clone());
        for (k, v) in current_bindings {
            if let Some(parent_value) = parent_scenario.bindings.as_ref().and_then(|pb| pb.get(&k))
            {
                if *parent_value != v {
                    return Err(YamlError::ParseScenarioConfigSourceError(
                        ParseScenarioConfigSourceError::ParentBindingShadowedError(k.to_string()),
                    ));
                }
            }
            bindings.insert(k.to_string(), v.to_string());
        }

        let runs = optional_string(scenario_yaml, "runs")
            .map(|runs| Scenario::validate_runs(&runs))
            .transpose()?;
        let blocks = optional_string(scenario_yaml, "blocks")
            .map(|blocks| Scenario::validate_blocks(&blocks))
            .transpose()?;

        if let Some(deployer_name) = optional_string(scenario_yaml, "deployer") {
            let current_deployer = Deployer::parse_from_yaml(document.clone(), &deployer_name)?;

            if let Some(parent_deployer) = parent_scenario.deployer.as_ref() {
                if current_deployer.label != parent_deployer.label {
                    return Err(YamlError::ParseScenarioConfigSourceError(
                        ParseScenarioConfigSourceError::ParentDeployerShadowedError(
                            deployer_name.clone(),
                        ),
                    ));
                }
            }

            *deployer = Some(Arc::new(current_deployer));
        }

        let key = if parent_scenario.key.is_empty() {
            scenario_key.clone()
        } else {
            format!("{}.{}", parent_scenario.key, scenario_key.clone())
        };
        scenarios.insert(
            key.clone(),
            Scenario {
                document: document.clone(),
                key: key.clone(),
                bindings: bindings.clone(),
                runs,
                blocks,
                deployer: deployer.clone().ok_or(
                    ParseScenarioConfigSourceError::DeployerNotFound(scenario_key),
                )?,
            },
        );

        if let Some(scenarios_yaml) = optional_hash(scenario_yaml, "scenarios") {
            for (child_key, child_scenario_yaml) in scenarios_yaml {
                let child_key = child_key.as_str().unwrap_or_default().to_string();
                Self::validate_scenario(
                    document.clone(),
                    deployer,
                    scenarios,
                    ScenarioParent {
                        key: key.clone(),
                        bindings: Some(bindings.clone()),
                        deployer: deployer.clone(),
                    },
                    child_key,
                    child_scenario_yaml,
                )?;
            }
        }

        Ok(())
    }

    pub fn update_bindings(
        &mut self,
        bindings: HashMap<String, String>,
    ) -> Result<Self, YamlError> {
        let scenario_parts = self.key.split('.').collect::<Vec<_>>();
        let base_scenario = scenario_parts[0];

        let mut new_bindings = Hash::new();
        for (k, v) in bindings {
            new_bindings.insert(StrictYaml::String(k), StrictYaml::String(v));
        }

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut scenarios)) =
                document_hash.get_mut(&StrictYaml::String("scenarios".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut scenario)) =
                    scenarios.get_mut(&StrictYaml::String(base_scenario.to_string()))
                {
                    if let Some(StrictYaml::Hash(ref mut base_bindings)) =
                        scenario.get_mut(&StrictYaml::String("bindings".to_string()))
                    {
                        let updates: Vec<_> = base_bindings
                            .keys()
                            .filter_map(|k| new_bindings.get(k).map(|v| (k.clone(), v.clone())))
                            .collect();
                        for (k, v) in updates {
                            base_bindings.insert(k, v);
                        }

                        let scenario_parts_vec: Vec<_> = scenario_parts.iter().skip(1).collect();
                        let mut current = scenario;

                        for &part in scenario_parts_vec {
                            let next_scenario =
                                if let Some(StrictYaml::Hash(ref mut sub_scenarios)) =
                                    current.get_mut(&StrictYaml::String("scenarios".to_string()))
                                {
                                    if let Some(StrictYaml::Hash(ref mut sub_scenario)) =
                                        sub_scenarios.get_mut(&StrictYaml::String(part.to_string()))
                                    {
                                        if let Some(StrictYaml::Hash(ref mut sub_bindings)) =
                                            sub_scenario.get_mut(&StrictYaml::String(
                                                "bindings".to_string(),
                                            ))
                                        {
                                            let sub_updates: Vec<_> = sub_bindings
                                                .keys()
                                                .filter_map(|k| {
                                                    new_bindings
                                                        .get(k)
                                                        .map(|v| (k.clone(), v.clone()))
                                                })
                                                .collect();

                                            for (k, v) in sub_updates {
                                                sub_bindings.insert(k, v);
                                            }
                                        } else {
                                            return Err(YamlError::ParseError(format!(
                                                "bindings not found in scenario {}",
                                                part
                                            )));
                                        }
                                        sub_scenario
                                    } else {
                                        return Err(YamlError::ParseError(format!(
                                            "{} not found in sub scenarios",
                                            part
                                        )));
                                    }
                                } else {
                                    return Err(YamlError::ParseError(format!(
                                        "scenarios not found for part {}",
                                        part
                                    )));
                                };
                            current = next_scenario;
                        }
                    } else {
                        return Err(YamlError::ParseError(format!(
                            "bindings not found in scenario {}",
                            base_scenario
                        )));
                    }
                } else {
                    return Err(YamlError::ParseError(format!(
                        "missing field: {} in scenarios",
                        base_scenario
                    )));
                }
            } else {
                return Err(YamlError::ParseError(
                    "missing field: scenarios".to_string(),
                ));
            }
        } else {
            return Err(YamlError::ParseError("document parse error".to_string()));
        }

        Ok(self.clone())
    }
}

impl YamlParsableHash for Scenario {
    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let scenarios_hash = require_hash(
            &document_read,
            Some("scenarios"),
            Some("missing field: scenarios".to_string()),
        )?;

        let mut scenarios = HashMap::new();

        for (key_yaml, scenario_yaml) in scenarios_hash {
            let scenario_key = key_yaml.as_str().unwrap_or_default().to_string();

            let mut deployer: Option<Arc<Deployer>> = None;

            Self::validate_scenario(
                document.clone(),
                &mut deployer,
                &mut scenarios,
                ScenarioParent {
                    key: "".to_string(),
                    bindings: None,
                    deployer: None,
                },
                scenario_key.clone(),
                scenario_yaml,
            )?;

            if deployer.is_none() {
                return Err(YamlError::ParseScenarioConfigSourceError(
                    ParseScenarioConfigSourceError::DeployerNotFound(scenario_key),
                ));
            }
        }

        Ok(scenarios)
    }
}

impl Default for Scenario {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: Arc::new(Deployer::default()),
        }
    }
}

impl PartialEq for Scenario {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.bindings == other.bindings
            && self.runs == other.runs
            && self.blocks == other.blocks
            && self.deployer == other.deployer
    }
}

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
    #[error("Failed to parse blocks: {0}")]
    BlocksParseError(String),
}

#[derive(Default)]
pub struct ScenarioParent {
    key: String,
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
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: name.clone(),
            bindings: bindings.clone(),
            runs: self.runs,
            blocks: self.blocks.clone(),
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
                        key: "".to_string(),
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
    use super::*;
    use crate::test::mock_deployer;
    use alloy::primitives::Address;
    use std::collections::HashMap;
    use url::Url;
    use yaml::tests::get_document;

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
            key: "".to_string(),
            bindings: Some(parent_bindings),
            deployer: Some(mock_deployer()),
        };

        let mut child_bindings = HashMap::new();
        child_bindings.insert("shared_key".to_string(), "child_value".to_string()); // Intentionally shadowing parent binding

        let child_scenario = ScenarioConfigSource {
            bindings: child_bindings,
            runs: None,
            blocks: None,
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

    #[test]
    fn test_parse_scenarios_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Scenario::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: scenarios".to_string())
        );

        let yaml = r#"
scenarios:
    scenario1:
        test: test
"#;
        let error = Scenario::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("bindings map missing in scenario: scenario1".to_string())
        );

        let yaml = r#"
scenarios:
    scenario1:
        bindings:
            key1:
                - value1
"#;
        let error = Scenario::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "binding value must be a string for key: key1 in scenario: scenario1".to_string()
            )
        );

        let yaml = r#"
scenarios:
    scenario1:
        bindings:
            key1:
                - value1: value2
"#;
        let error = Scenario::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "binding value must be a string for key: key1 in scenario: scenario1".to_string()
            )
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
deployers:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
scenarios:
    scenario1:
        deployer: mainnet
        bindings:
            key1: some-value
        scenarios:
            scenario2:
                bindings:
                    key1: value
"#;
        let error = Scenario::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error.to_string(),
            YamlError::ParseScenarioConfigSourceError(
                ParseScenarioConfigSourceError::ParentBindingShadowedError("key1".to_string())
            )
            .to_string()
        );
    }
}
