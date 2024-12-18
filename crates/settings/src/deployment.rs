use crate::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct Deployment {
    #[typeshare(typescript(type = "Scenario"))]
    pub scenario: Arc<Scenario>,
    #[typeshare(typescript(type = "Order"))]
    pub order: Arc<Order>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Deployment);

#[derive(Error, Debug, PartialEq)]
pub enum ParseDeploymentConfigSourceError {
    #[error("Scenario not found: {0}")]
    ScenarioNotFoundError(String),
    #[error("Order not found: {0}")]
    OrderNotFoundError(String),
    #[error("Scenario and Order do not match")]
    NoMatch,
}

impl DeploymentConfigSource {
    pub fn try_into_deployment(
        self,
        scenarios: &HashMap<String, Arc<Scenario>>,
        orders: &HashMap<String, Arc<Order>>,
    ) -> Result<Deployment, ParseDeploymentConfigSourceError> {
        let scenario = scenarios
            .get(&self.scenario)
            .ok_or(ParseDeploymentConfigSourceError::ScenarioNotFoundError(
                self.scenario.clone(),
            ))
            .map(Arc::clone)?;

        let order = orders
            .get(&self.order)
            .ok_or(ParseDeploymentConfigSourceError::OrderNotFoundError(
                self.order.clone(),
            ))
            .map(Arc::clone)?;

        // check validity
        if let Some(deployer) = &order.deployer {
            if deployer != &scenario.deployer {
                return Err(ParseDeploymentConfigSourceError::NoMatch);
            }
        };

        Ok(Deployment { scenario, order })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;
    use std::sync::RwLock;
    use strict_yaml_rust::StrictYaml;

    #[test]
    fn test_try_into_deployment_success() {
        let order_name = "order1";
        let scenario_name = "scenario1";
        let scenario = Scenario {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "scenario1".into(),
            bindings: HashMap::new(),
            deployer: mock_deployer(),
            runs: None,
            blocks: None,
        };
        let order = Order {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            inputs: vec![],
            outputs: vec![],
            network: mock_network(),
            deployer: None,
            orderbook: None,
        };
        let orders = HashMap::from([(order_name.to_string(), Arc::new(order))]);
        let scenarios = HashMap::from([(scenario_name.to_string(), Arc::new(scenario))]);
        let deploment_string = DeploymentConfigSource {
            scenario: scenario_name.to_string(),
            order: order_name.to_string(),
        };
        let result = deploment_string.try_into_deployment(&scenarios, &orders);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_into_deployment_error() {
        let order_name = "order1";
        let scenario_name = "scenario1";
        let other_scenario_name = "scenario2";
        let scenario = Scenario {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "scenario1".into(),
            bindings: HashMap::new(),
            deployer: mock_deployer(),
            runs: None,
            blocks: None,
        };
        let order = Order {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            inputs: vec![],
            outputs: vec![],
            network: mock_network(),
            deployer: None,
            orderbook: None,
        };
        let orders = HashMap::from([(order_name.to_string(), Arc::new(order))]);
        let scenarios = HashMap::from([(scenario_name.to_string(), Arc::new(scenario))]);
        let deploment_string = DeploymentConfigSource {
            scenario: other_scenario_name.to_string(),
            order: order_name.to_string(),
        };
        let result = deploment_string.try_into_deployment(&scenarios, &orders);
        assert!(matches!(
            result,
            Err(ParseDeploymentConfigSourceError::ScenarioNotFoundError(_))
        ));
    }
}
