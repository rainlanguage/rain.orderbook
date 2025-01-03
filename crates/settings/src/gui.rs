use crate::{Deployment, Token, TokenRef};
use alloy::primitives::{ruint::ParseError, utils::UnitsError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

// Config source for Gui
#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiPresetSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: String,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(GuiPresetSource);

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiDepositSource {
    pub token: TokenRef,
    pub presets: Vec<String>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiFieldDefinitionSource {
    pub binding: String,
    pub name: String,
    pub description: Option<String>,
    pub presets: Option<Vec<GuiPresetSource>>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiDeploymentSource {
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDepositSource>,
    pub fields: Vec<GuiFieldDefinitionSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_tokens: Option<Vec<TokenRef>>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiConfigSource {
    pub name: String,
    pub description: String,
    pub deployments: HashMap<String, GuiDeploymentSource>,
}
impl GuiConfigSource {
    pub fn try_into_gui(
        self,
        deployments: &HashMap<String, Arc<Deployment>>,
        tokens: &HashMap<String, Arc<Token>>,
    ) -> Result<Gui, ParseGuiConfigSourceError> {
        let gui_deployments = self
            .deployments
            .iter()
            .map(|(deployment_name, deployment_source)| {
                let deployment = deployments
                    .get(deployment_name)
                    .ok_or(ParseGuiConfigSourceError::DeploymentNotFoundError(
                        deployment_name.clone(),
                    ))
                    .map(Arc::clone)?;

                let deposits = deployment_source
                    .deposits
                    .iter()
                    .map(|deposit_source| {
                        let token = tokens
                            .get(&deposit_source.token)
                            .ok_or(ParseGuiConfigSourceError::TokenNotFoundError(
                                deposit_source.token.clone(),
                            ))
                            .map(Arc::clone)?;

                        Ok(GuiDeposit {
                            token: token.clone(),
                            token_name: deposit_source.token.clone(),
                            presets: deposit_source.presets.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                let fields = deployment_source
                    .fields
                    .iter()
                    .map(|field_source| {
                        Ok(GuiFieldDefinition {
                            binding: field_source.binding.clone(),
                            name: field_source.name.clone(),
                            description: field_source.description.clone(),
                            presets: field_source
                                .presets
                                .as_ref()
                                .map(|presets| {
                                    presets
                                        .iter()
                                        .enumerate()
                                        .map(|(i, preset)| {
                                            Ok(GuiPreset {
                                                id: i.to_string(),
                                                name: preset.name.clone(),
                                                value: preset.value.clone(),
                                            })
                                        })
                                        .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()
                                })
                                .transpose()?,
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                Ok((
                    deployment_name.clone(),
                    GuiDeployment {
                        deployment,
                        deployment_name: deployment_name.to_string(),
                        name: deployment_source.name.clone(),
                        description: deployment_source.description.clone(),
                        deposits,
                        fields,
                        select_tokens: deployment_source.select_tokens.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, ParseGuiConfigSourceError>>()?;

        Ok(Gui {
            name: self.name,
            description: self.description,
            deployments: gui_deployments,
        })
    }
}

#[derive(Error, Debug)]
pub enum ParseGuiConfigSourceError {
    #[error("Deployment not found: {0}")]
    DeploymentNotFoundError(String),
    #[error("Token not found: {0}")]
    TokenNotFoundError(String),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    UnitsError(#[from] UnitsError),
}

// Config for Gui

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiPreset {
    pub id: String,
    #[typeshare(typescript(type = "string"))]
    pub name: Option<String>,
    pub value: String,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(GuiPreset);

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiDeposit {
    #[typeshare(typescript(type = "Token"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "Erc20"))]
    pub token: Arc<Token>,
    pub token_name: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string[]"))]
    pub presets: Vec<String>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(GuiDeposit);

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiDeployment {
    #[typeshare(typescript(type = "Deployment"))]
    pub deployment: Arc<Deployment>,
    pub deployment_name: String,
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDeposit>,
    pub fields: Vec<GuiFieldDefinition>,
    pub select_tokens: Option<Vec<String>>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(GuiDeployment);

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiFieldDefinition {
    pub binding: String,
    pub name: String,
    pub description: Option<String>,
    pub presets: Option<Vec<GuiPreset>>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(GuiFieldDefinition);

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct Gui {
    pub name: String,
    pub description: String,
    pub deployments: HashMap<String, GuiDeployment>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Gui);

#[cfg(test)]
mod tests {
    use std::sync::RwLock;

    use super::*;
    use crate::{
        test::{mock_deployer, mock_network, mock_token},
        Order, Scenario,
    };
    use alloy::primitives::Address;
    use strict_yaml_rust::StrictYaml;

    #[test]
    fn test_gui_creation_success() {
        let gui_config_source = GuiConfigSource {
            name: "test-gui".to_string(),
            description: "test-gui-description".to_string(),
            deployments: HashMap::from([(
                "test-deployment".to_string(),
                GuiDeploymentSource {
                    name: "test-deployment".to_string(),
                    description: "test-deployment-description".to_string(),
                    deposits: vec![GuiDepositSource {
                        token: "test-token".to_string(),
                        presets: vec!["1.3".to_string(), "2.7".to_string()],
                    }],
                    fields: vec![
                        GuiFieldDefinitionSource {
                            binding: "test-binding".to_string(),
                            name: "test-name".to_string(),
                            description: Some("test-description".to_string()),
                            presets: Some(vec![
                                GuiPresetSource {
                                    name: Some("test-preset".to_string()),
                                    value: "0.015".to_string(),
                                },
                                GuiPresetSource {
                                    name: Some("test-preset-2".to_string()),
                                    value: "0.3".to_string(),
                                },
                            ]),
                        },
                        GuiFieldDefinitionSource {
                            binding: "test-binding-2".to_string(),
                            name: "test-name-2".to_string(),
                            description: Some("test-description-2".to_string()),
                            presets: Some(vec![
                                GuiPresetSource {
                                    name: None,
                                    value: "3.2".to_string(),
                                },
                                GuiPresetSource {
                                    name: None,
                                    value: "4.8".to_string(),
                                },
                            ]),
                        },
                        GuiFieldDefinitionSource {
                            binding: "test-binding-3".to_string(),
                            name: "test-name-3".to_string(),
                            description: Some("test-description-3".to_string()),
                            presets: Some(vec![
                                GuiPresetSource {
                                    name: None,
                                    value: Address::default().to_string(),
                                },
                                GuiPresetSource {
                                    name: None,
                                    value: "some-value".to_string(),
                                },
                                GuiPresetSource {
                                    name: None,
                                    value: "true".to_string(),
                                },
                            ]),
                        },
                    ],
                    select_tokens: Some(vec!["test-token".to_string()]),
                },
            )]),
        };
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
        let deployment = Deployment {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-deployment".to_string(),
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };
        let deployments = HashMap::from([("test-deployment".to_string(), Arc::new(deployment))]);
        let tokens = HashMap::from([("test-token".to_string(), mock_token("test-token"))]);

        let gui = gui_config_source
            .try_into_gui(&deployments, &tokens)
            .unwrap();

        assert_eq!(gui.name, "test-gui");
        assert_eq!(gui.description, "test-gui-description");
        assert_eq!(gui.deployments.len(), 1);
        let deployment = &gui.deployments.get("test-deployment").unwrap();
        assert_eq!(deployment.name, "test-deployment");
        assert_eq!(deployment.description, "test-deployment-description");
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = &deployment.deposits[0];
        assert_eq!(deposit.token.label, Some("test-token".to_string()));
        assert_eq!(deposit.presets.len(), 2);
        assert_eq!(deposit.presets[0], "1.3".to_string());
        assert_eq!(deposit.presets[1], "2.7".to_string());
        assert_eq!(deployment.fields.len(), 3);
        let field1 = &deployment.fields[0];
        assert_eq!(field1.binding, "test-binding");
        assert_eq!(field1.name, "test-name");
        assert_eq!(field1.description, Some("test-description".to_string()));
        let presets = field1.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, Some("test-preset".to_string()));
        assert_eq!(presets[0].value, "0.015".to_string());
        assert_eq!(presets[1].name, Some("test-preset-2".to_string()));
        assert_eq!(presets[1].value, "0.3".to_string());
        let field2 = &deployment.fields[1];
        assert_eq!(field2.binding, "test-binding-2");
        assert_eq!(field2.name, "test-name-2");
        assert_eq!(field2.description, Some("test-description-2".to_string()));
        let presets = field2.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, None);
        assert_eq!(presets[1].name, None);
        assert_eq!(presets[1].value, "4.8".to_string());
        let field3 = &deployment.fields[2];
        assert_eq!(field3.binding, "test-binding-3");
        assert_eq!(field3.name, "test-name-3");
        assert_eq!(field3.description, Some("test-description-3".to_string()));
        let presets = field3.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].value, Address::default().to_string());
        assert_eq!(presets[1].value, "some-value".to_string());
        assert_eq!(presets[2].value, "true".to_string());
        assert_eq!(
            deployment.select_tokens,
            Some(vec!["test-token".to_string()])
        );
    }
}
