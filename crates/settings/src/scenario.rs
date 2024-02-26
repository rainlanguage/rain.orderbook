use crate::*;
use std::{collections::HashMap, num::ParseIntError, sync::Arc};
use thiserror::Error;

#[derive(Debug)]
pub struct Scenario {
    pub bindings: HashMap<String, String>,
    pub runs: Option<u64>,
    pub network: Option<Arc<Network>>, // defaults to network of the same name
    pub deployer: Option<Arc<Deployer>>, // defaults to deployer of the same name
    pub orderbook: Option<Arc<Orderbook>>, // defaults to orderbook of the same name
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseScenarioStringError {
    #[error("Failed to parse deployer")]
    DeployerParseError(ParseDeployerStringError),
    #[error("Network not found: {0}")]
    NetworkNotFoundError(String),
    #[error("Failed to parse orderbook")]
    OrderbookParseError(ParseOrderbookStringError),
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

// Nested scenarios within the ScenarioString struct are flattened out into a
// hashmap of scenarios, where the key is the path such as foo.bar.baz.
// Every level of the scenario path inherits its parents bindings recursively.
// Shadowing is disallowed for networks, deployers, orderbooks and specific bindings.
// If a child specifies one that is already set by the parent, this is an error.
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
        let network_ref = match &self.network {
            Some(network_name) => networks.get(network_name),
            None => networks.get(&name),
        }
        .map(Arc::clone);

        if let Some(parent_network) = parent_network {
            if let Some(network_ref) = &network_ref {
                if network_ref != parent_network {
                    return Err(ParseScenarioStringError::ParentNetworkShadowedError(name));
                }
            }
        }

        let deployer_ref = match &self.deployer {
            Some(deployer_name) => deployers.get(deployer_name),
            None => deployers.get(&name),
        }
        .map(Arc::clone);

        if let Some(parent_deployer) = parent_deployer {
            if let Some(deployer_ref) = &deployer_ref {
                if deployer_ref != parent_deployer {
                    return Err(ParseScenarioStringError::ParentDeployerShadowedError(name));
                }
            }
        }

        let orderbook_ref = match &self.orderbook {
            Some(orderbook_name) => orderbooks.get(orderbook_name),
            None => orderbooks.get(&name),
        }
        .map(Arc::clone);

        if let Some(parent_orderbook) = parent_orderbook {
            if let Some(orderbook_ref) = &orderbook_ref {
                if orderbook_ref != parent_orderbook {
                    return Err(ParseScenarioStringError::ParentOrderbookShadowedError(name));
                }
            }
        }

        // merge bindings and check for shadowing
        let mut bindings = parent_bindings.map(|pb| pb.clone()).unwrap_or_default();

        for (k, v) in &self.bindings {
            if let Some(parent_bindings) = parent_bindings {
                if let Some(parent_value) = parent_bindings.get(k) {
                    if parent_value != v {
                        return Err(ParseScenarioStringError::ParentBindingShadowedError(
                            k.to_string(),
                        ));
                    }
                }
            }
            bindings.insert(k.to_string(), v.to_string());
        }

        let parent_scenario = Arc::new(Scenario {
            bindings: bindings.clone(),
            runs: self
                .clone()
                .runs
                .map(|s| s.parse().map_err(ParseScenarioStringError::RunsParseError))
                .transpose()?,
            network: network_ref.clone(),
            deployer: deployer_ref.clone(),
            orderbook: orderbook_ref.clone(),
        });

        let mut scenarios = HashMap::new();

        scenarios.insert(name.clone(), parent_scenario);

        if let Some(scenarios_map) = &self.scenarios {
            for (child_name, child_scenario) in scenarios_map {
                let child_scenarios = child_scenario.try_into_scenarios(
                    child_name.to_string(),
                    Some(&bindings),
                    network_ref.as_ref(),
                    deployer_ref.as_ref(),
                    orderbook_ref.as_ref(),
                    networks,
                    deployers,
                    orderbooks,
                )?;

                for (child_name, child_scenario) in child_scenarios {
                    scenarios.insert(format!("{}.{}", name, child_name), child_scenario);
                }
            }
        }

        Ok(scenarios)
    }
}
