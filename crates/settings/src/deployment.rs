use crate::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deployment {
    #[typeshare(typescript(type = "Scenario"))]
    pub scenario: Arc<Scenario>,
    #[typeshare(typescript(type = "Order"))]
    pub order: Arc<Order>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseDeploymentStringError {
    #[error("Scenario not found: {0}")]
    ScenarioNotFoundError(String),
    #[error("Order not found: {0}")]
    OrderNotFoundError(String),
    #[error("Scenario and Order do not match")]
    NoMatch,
}

impl DeploymentString {
    pub fn try_into_deployment(
        self,
        scenarios: &HashMap<String, Arc<Scenario>>,
        orders: &HashMap<String, Arc<Order>>,
    ) -> Result<Deployment, ParseDeploymentStringError> {
        let scenario = scenarios
            .get(&self.scenario)
            .ok_or(ParseDeploymentStringError::ScenarioNotFoundError(
                self.scenario.clone(),
            ))
            .map(Arc::clone)?;

        let order = orders
            .get(&self.order)
            .ok_or(ParseDeploymentStringError::ScenarioNotFoundError(
                self.order.clone(),
            ))
            .map(Arc::clone)?;

        // check validity
        if let Some(deployer) = &order.deployer {
            if deployer != &scenario.deployer {
                return Err(ParseDeploymentStringError::NoMatch);
            }
        };

        Ok(Deployment { scenario, order })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    // Mock a simple Network struct for testing purposes
    #[derive(Debug, Clone)]
    struct MockNetwork {
        name: String,
    }

    impl PartialEq for MockNetwork {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }

    #[test]
    fn test_try_into_deployment_success() {
        let order_name = "order1";
        let scenario_name = "scenario1";
        let scenario = Scenario {
            bindings: HashMap::new(),
            deployer: mock_deployer(),
            runs: None,
        };
        let order = Order {
            inputs: vec![],
            outputs: vec![],
            network: mock_network(),
            deployer: None,
            orderbook: None,
        };
        let orders = HashMap::from([(order_name.to_string(), Arc::new(order))]);
        let scenarios = HashMap::from([(scenario_name.to_string(), Arc::new(scenario))]);
        let deploment_string = DeploymentString {
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
            bindings: HashMap::new(),
            deployer: mock_deployer(),
            runs: None,
        };
        let order = Order {
            inputs: vec![],
            outputs: vec![],
            network: mock_network(),
            deployer: None,
            orderbook: None,
        };
        let orders = HashMap::from([(order_name.to_string(), Arc::new(order))]);
        let scenarios = HashMap::from([(scenario_name.to_string(), Arc::new(scenario))]);
        let deploment_string = DeploymentString {
            scenario: other_scenario_name.to_string(),
            order: order_name.to_string(),
        };
        let result = deploment_string.try_into_deployment(&scenarios, &orders);
        assert!(matches!(
            result,
            Err(ParseDeploymentStringError::ScenarioNotFoundError(_))
        ));
    }
}
