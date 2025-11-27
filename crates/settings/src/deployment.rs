use crate::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::{
    context::{Context, GuiContextTrait},
    default_document, require_hash, require_string, FieldErrorKind, YamlError, YamlParsableHash,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DeploymentCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub scenario: Arc<ScenarioCfg>,
    pub order: Arc<OrderCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentCfg);

impl DeploymentCfg {
    pub fn parse_order_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployment_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(deployments_hash) = require_hash(&document_read, Some("deployments"), None) {
                if let Some(deployment_yaml) =
                    deployments_hash.get(&StrictYaml::String(deployment_key.to_string()))
                {
                    return require_string(deployment_yaml, Some("order"), None);
                }
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing("order".to_string()),
            location: format!("deployment '{deployment_key}'"),
        })
    }
}

impl YamlParsableHash for DeploymentCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut deployments = HashMap::new();

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(deployments_hash) = require_hash(&document_read, Some("deployments"), None) {
                for (key_yaml, deployment_yaml) in deployments_hash {
                    let deployment_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("deployment '{deployment_key}'");

                    if let Some(context) = context {
                        if let Some(current_deployment) = context.get_current_deployment() {
                            if current_deployment != &deployment_key {
                                continue;
                            }
                        }
                    }

                    let mut context = Context::from_context(context);

                    let order_key =
                        require_string(deployment_yaml, Some("order"), Some(location.clone()))?;
                    context.add_current_order(order_key.clone());

                    let order =
                        OrderCfg::parse_from_yaml(documents.clone(), &order_key, Some(&context))?;
                    context.add_order(Arc::new(order.clone()));

                    let scenario_key =
                        require_string(deployment_yaml, Some("scenario"), Some(location.clone()))?;
                    let scenario = ScenarioCfg::parse_from_yaml(
                        documents.clone(),
                        &scenario_key,
                        Some(&context),
                    )?;

                    if let Some(deployer) = &order.deployer {
                        if deployer != &scenario.deployer {
                            return Err(YamlError::ParseDeploymentConfigSourceError(
                                ParseDeploymentConfigSourceError::NoMatch,
                            ));
                        }
                    }

                    let deployment = DeploymentCfg {
                        document: document.clone(),
                        key: deployment_key.clone(),
                        scenario: Arc::new(scenario),
                        order: Arc::new(order),
                    };

                    if deployments.contains_key(&deployment_key) {
                        return Err(YamlError::KeyShadowing(
                            deployment_key.clone(),
                            "deployments".to_string(),
                        ));
                    }
                    deployments.insert(deployment_key, deployment);
                }
            }
        }

        if let Some(context) = context {
            if let Some(current_deployment) = context.get_current_deployment() {
                if !deployments.contains_key(current_deployment) {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(current_deployment.to_string()),
                        location: "deployments".to_string(),
                    });
                }
            }
        }
        if deployments.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(deployments)
    }

    fn to_yaml_value(&self) -> Result<StrictYaml, YamlError> {
        let mut deployment_yaml = Hash::new();

        deployment_yaml.insert(
            StrictYaml::String("scenario".to_string()),
            StrictYaml::String(self.scenario.key.clone()),
        );

        deployment_yaml.insert(
            StrictYaml::String("order".to_string()),
            StrictYaml::String(self.order.key.clone()),
        );

        Ok(StrictYaml::Hash(deployment_yaml))
    }
}

impl Default for DeploymentCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            scenario: Arc::new(ScenarioCfg::default()),
            order: Arc::new(OrderCfg::default()),
        }
    }
}

impl PartialEq for DeploymentCfg {
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

impl ParseDeploymentConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseDeploymentConfigSourceError::ScenarioNotFoundError(scenario) =>
                format!("The scenario '{}' referenced in your deployment configuration was not found in your YAML configuration. Please check that this scenario is defined correctly.", scenario),
            ParseDeploymentConfigSourceError::OrderNotFoundError(order) =>
                format!("The order '{}' referenced in your deployment configuration was not found in your YAML configuration. Please check that this order is defined correctly.", order),
            ParseDeploymentConfigSourceError::NoMatch =>
                "The scenario and order in your deployment configuration do not match. The deployer specified in the order must match the deployer specified in the scenario.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{mock_deployer, mock_network, mock_orderbook};
    use yaml::tests::get_document;

    #[test]
    fn test_parse_deployments_from_yaml() {
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
deployers:
    deployer1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        deployer: deployer1
test: test
"#;
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'deployments' in root"
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
deployers:
    deployer1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        deployer: deployer1
deployments:
    deployment1:
        test: test
        "#;
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'order' in deployment 'deployment1'"
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
deployers:
    deployer1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        deployer: deployer1
deployments:
    deployment1:
        order: order1
        test: test
        "#;
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("scenario".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'scenario' in deployment 'deployment1'"
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
    network2:
        rpcs:
            - https://test.com
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
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error.to_string(),
            YamlError::ParseDeploymentConfigSourceError(ParseDeploymentConfigSourceError::NoMatch)
                .to_string()
        );
        assert_eq!(
            error.to_readable_msg(),
            "Deployment configuration error in your YAML: The scenario and order in your deployment configuration do not match. The deployer specified in the order must match the deployer specified in the scenario."
        );
    }

    const PREFIX: &str = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
    network2:
        rpcs:
            - https://test.com
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
    scenario2:
        bindings:
            test: test
        deployer: deployer2
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000000
        network: network2
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        deployer: deployer1
    order2:
        inputs:
            - token: token2
        outputs:
            - token: token2
        deployer: deployer2
"#;

    #[test]
    fn test_parse_deployments_from_yaml_multiple_files() {
        let yaml_one = r#"
deployments:
    DeploymentOne:
        scenario: scenario1
        order: order1
"#;
        let yaml_two = r#"
deployments:
    DeploymentTwo:
        scenario: scenario2
        order: order2
"#;

        let deployments = DeploymentCfg::parse_all_from_yaml(
            vec![
                get_document(&format!("{PREFIX}{yaml_one}")),
                get_document(yaml_two),
            ],
            None,
        )
        .unwrap();

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
        scenario: scenario1
        order: order1
"#;
        let yaml_two = r#"
deployments:
    DuplicateDeployment:
        scenario: scenario2
        order: order2
"#;

        let error = DeploymentCfg::parse_all_from_yaml(
            vec![
                get_document(&format!("{PREFIX}{yaml_one}")),
                get_document(yaml_two),
            ],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateDeployment".to_string(), "deployments".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'DuplicateDeployment' is defined multiple times in your YAML configuration at deployments"
        );
    }

    #[test]
    fn test_parse_order_key() {
        let yaml = r#"
deployments: test
"#;
        let error =
            DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'order' in deployment 'deployment1'"
        );

        let yaml = r#"
deployments:
  - test
"#;
        let error =
            DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'order' in deployment 'deployment1'"
        );

        let yaml = r#"
deployments:
  - test: test
"#;
        let error =
            DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'order' in deployment 'deployment1'"
        );

        let yaml = r#"
deployments:
  deployment1:
    order: order1
    scenario: scenario1
"#;
        let res = DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap();
        assert_eq!(res, "order1");
    }

    #[test]
    fn test_to_yaml_hash_serializes_deployments() {
        let deployer = mock_deployer();
        let network = mock_network();

        let deployment_with_optionals = DeploymentCfg {
            document: default_document(),
            key: "with-optionals".to_string(),
            scenario: Arc::new(ScenarioCfg {
                document: default_document(),
                key: "scenario-with-runs".to_string(),
                bindings: HashMap::new(),
                runs: Some(3),
                blocks: None,
                deployer: deployer.clone(),
            }),
            order: Arc::new(OrderCfg {
                document: default_document(),
                key: "order-with-orderbook".to_string(),
                inputs: vec![],
                outputs: vec![],
                network: network.clone(),
                deployer: Some(deployer.clone()),
                orderbook: Some(mock_orderbook()),
            }),
        };

        let deployment_without_optionals = DeploymentCfg {
            document: default_document(),
            key: "without-optionals".to_string(),
            scenario: Arc::new(ScenarioCfg {
                document: default_document(),
                key: "scenario-no-optionals".to_string(),
                bindings: HashMap::new(),
                runs: None,
                blocks: None,
                deployer: deployer.clone(),
            }),
            order: Arc::new(OrderCfg {
                document: default_document(),
                key: "order-no-optionals".to_string(),
                inputs: vec![],
                outputs: vec![],
                network: network.clone(),
                deployer: None,
                orderbook: None,
            }),
        };

        let deployments = HashMap::from([
            (
                deployment_with_optionals.key.clone(),
                deployment_with_optionals,
            ),
            (
                deployment_without_optionals.key.clone(),
                deployment_without_optionals,
            ),
        ]);

        let yaml = DeploymentCfg::to_yaml_hash(&deployments).unwrap();

        let StrictYaml::Hash(deployments_hash) = yaml else {
            panic!("deployments were not serialized to a YAML hash");
        };

        let with_optionals_hash = deployments_hash
            .get(&StrictYaml::String("with-optionals".to_string()))
            .and_then(|value| value.as_hash())
            .expect("with-optionals deployment missing or not a map");
        let without_optionals_hash = deployments_hash
            .get(&StrictYaml::String("without-optionals".to_string()))
            .and_then(|value| value.as_hash())
            .expect("without-optionals deployment missing or not a map");

        assert_eq!(
            with_optionals_hash.get(&StrictYaml::String("scenario".to_string())),
            Some(&StrictYaml::String("scenario-with-runs".to_string()))
        );
        assert_eq!(
            with_optionals_hash.get(&StrictYaml::String("order".to_string())),
            Some(&StrictYaml::String("order-with-orderbook".to_string()))
        );
        assert_eq!(
            without_optionals_hash.get(&StrictYaml::String("scenario".to_string())),
            Some(&StrictYaml::String("scenario-no-optionals".to_string()))
        );
        assert_eq!(
            without_optionals_hash.get(&StrictYaml::String("order".to_string())),
            Some(&StrictYaml::String("order-no-optionals".to_string()))
        );

        assert_eq!(with_optionals_hash.len(), 2);
        assert_eq!(without_optionals_hash.len(), 2);
    }
}
