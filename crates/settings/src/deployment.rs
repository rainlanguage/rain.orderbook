use crate::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use typeshare::typeshare;
use yaml::{default_document, require_hash, require_string, YamlError, YamlParsableHash};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct Deployment {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "Scenario"))]
    pub scenario: Arc<Scenario>,
    #[typeshare(typescript(type = "Order"))]
    pub order: Arc<Order>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Deployment);

impl YamlParsableHash for Deployment {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut deployments = HashMap::new();

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            let deployments_hash = require_hash(
                &document_read,
                Some("deployments"),
                Some("missing field: deployments".to_string()),
            )?;

            for (key_yaml, deployment_yaml) in deployments_hash {
                let deployment_key = key_yaml.as_str().unwrap_or_default().to_string();

                let scenario = Scenario::parse_from_yaml(
                    documents.clone(),
                    &require_string(
                        deployment_yaml,
                        Some("scenario"),
                        Some(format!(
                            "scenario string missing in deployment: {deployment_key}"
                        )),
                    )?,
                )?;

                let order = Order::parse_from_yaml(
                    documents.clone(),
                    &require_string(
                        deployment_yaml,
                        Some("order"),
                        Some(format!(
                            "order string missing in deployment: {deployment_key}"
                        )),
                    )?,
                )?;

                if let Some(deployer) = &order.deployer {
                    if deployer != &scenario.deployer {
                        return Err(YamlError::ParseDeploymentConfigSourceError(
                            ParseDeploymentConfigSourceError::NoMatch,
                        ));
                    }
                }

                let deployment = Deployment {
                    document: document.clone(),
                    key: deployment_key.clone(),
                    scenario: Arc::new(scenario),
                    order: Arc::new(order),
                };

                if deployments.contains_key(&deployment_key) {
                    return Err(YamlError::KeyShadowing(deployment_key));
                }
                deployments.insert(deployment_key, deployment);
            }
        }

        Ok(deployments)
    }
}

impl Default for Deployment {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            scenario: Arc::new(Scenario::default()),
            order: Arc::new(Order::default()),
        }
    }
}

impl PartialEq for Deployment {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.scenario == other.scenario && self.order == other.order
    }
}

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

        Ok(Deployment {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: scenario.key.clone(),
            scenario,
            order,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;
    use std::sync::RwLock;
    use strict_yaml_rust::StrictYaml;
    use yaml::tests::get_document;

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

    #[test]
    fn test_parse_deployments_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Deployment::parse_all_from_yaml(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: deployments".to_string())
        );

        let yaml = r#"
deployments:
    deployment1:
        test: test
        "#;
        let error = Deployment::parse_all_from_yaml(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("scenario string missing in deployment: deployment1".to_string())
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
deployers:
    deployer1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        deployer: deployer1
deployments:
    deployment1:
        scenario: scenario1
        test: test
        "#;
        let error = Deployment::parse_all_from_yaml(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("order string missing in deployment: deployment1".to_string())
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
    network2:
        rpc: https://test.com
        chain-id: 2
deployers:
    deployer1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
    deployer2:
        address: 0x0000000000000000000000000000000000000000
        network: network2
scenarios:
    scenario1:
        bindings:
            test: test
        deployer: deployer1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network2
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        deployer: deployer2
deployments:
    deployment1:
        scenario: scenario1
        order: order1
        "#;
        let error = Deployment::parse_all_from_yaml(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error.to_string(),
            YamlError::ParseDeploymentConfigSourceError(ParseDeploymentConfigSourceError::NoMatch)
                .to_string()
        );
    }

    #[test]
    fn test_parse_deployments_from_yaml_multiple_files() {
        let yaml_one = r#"
deployments:
    DeploymentOne:
        scenario: ScenarioOne
        order: OrderOne
"#;
        let yaml_two = r#"
deployments:
    DeploymentTwo:
        scenario: ScenarioTwo
        order: OrderTwo
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let deployments = Deployment::parse_all_from_yaml(documents).unwrap();

        assert_eq!(deployments.len(), 2);
        assert!(deployments.contains_key("DeploymentOne"));
        assert!(deployments.contains_key("DeploymentTwo"));

        assert_eq!(
            deployments.get("DeploymentOne").unwrap().key,
            "DeploymentOne"
        );
        assert_eq!(
            deployments.get("DeploymentTwo").unwrap().key,
            "DeploymentTwo"
        );
    }

    #[test]
    fn test_parse_deployments_from_yaml_duplicate_key() {
        let yaml_one = r#"
deployments:
    DuplicateDeployment:
        scenario: ScenarioOne
        order: OrderOne
"#;
        let yaml_two = r#"
deployments:
    DuplicateDeployment:
        scenario: ScenarioTwo
        order: OrderTwo
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = Deployment::parse_all_from_yaml(documents).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateDeployment".to_string())
        );
    }
}
