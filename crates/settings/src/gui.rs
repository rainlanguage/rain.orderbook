use crate::{Deployment, DeploymentRef, Token, TokenRef};
use alloy::primitives::{ruint::ParseError, utils::UnitsError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::impl_wasm_traits;
#[cfg(target_family = "wasm")]
use serde_wasm_bindgen::{from_value, to_value};
#[cfg(target_family = "wasm")]
use tsify::Tsify;
#[cfg(target_family = "wasm")]
use wasm_bindgen::convert::{
    js_value_vector_from_abi, js_value_vector_into_abi, FromWasmAbi, IntoWasmAbi,
    LongRefFromWasmAbi, RefFromWasmAbi, TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
};
#[cfg(target_family = "wasm")]
use wasm_bindgen::describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR};
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsValue, UnwrapThrowExt};

// Config source for Gui

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(
    target_family = "wasm",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "kebab-case")]
pub struct GuiPresetSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: String,
}

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
    pub description: String,
    pub presets: Vec<GuiPresetSource>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiDeploymentSource {
    pub deployment: DeploymentRef,
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDepositSource>,
    pub fields: Vec<GuiFieldDefinitionSource>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiConfigSource {
    pub name: String,
    pub description: String,
    pub deployments: Vec<GuiDeploymentSource>,
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
            .map(|deployment_source| {
                let deployment = deployments
                    .get(&deployment_source.deployment)
                    .ok_or(ParseGuiConfigSourceError::DeploymentNotFoundError(
                        deployment_source.deployment.clone(),
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
                                .iter()
                                .map(|preset| {
                                    Ok(GuiPreset {
                                        name: preset.name.clone(),
                                        value: preset.value.clone(),
                                    })
                                })
                                .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                Ok(GuiDeployment {
                    deployment,
                    deployment_name: deployment_source.deployment.clone(),
                    name: deployment_source.name.clone(),
                    description: deployment_source.description.clone(),
                    deposits,
                    fields,
                })
            })
            .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

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
#[cfg_attr(
    target_family = "wasm",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct GuiPreset {
    name: Option<String>,
    value: String,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(
    target_family = "wasm",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct GuiDeposit {
    #[typeshare(typescript(type = "Token"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "Erc20"))]
    pub token: Arc<Token>,
    pub token_name: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string[]"))]
    pub presets: Vec<String>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(
    target_family = "wasm",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct GuiDeployment {
    #[typeshare(typescript(type = "Deployment"))]
    pub deployment: Arc<Deployment>,
    pub deployment_name: String,
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDeposit>,
    pub fields: Vec<GuiFieldDefinition>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(
    target_family = "wasm",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct GuiFieldDefinition {
    pub binding: String,
    pub name: String,
    pub description: String,
    pub presets: Vec<GuiPreset>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiFieldDefinition);

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(
    target_family = "wasm",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Gui {
    pub name: String,
    pub description: String,
    pub deployments: Vec<GuiDeployment>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{mock_deployer, mock_network, mock_token},
        Order, Scenario,
    };
    use alloy::primitives::Address;

    #[test]
    fn test_gui_creation_success() {
        let gui_config_source = GuiConfigSource {
            name: "test-gui".to_string(),
            description: "test-gui-description".to_string(),
            deployments: vec![GuiDeploymentSource {
                deployment: "test-deployment".to_string(),
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
                        description: "test-description".to_string(),
                        presets: vec![
                            GuiPresetSource {
                                name: Some("test-preset".to_string()),
                                value: "0.015".to_string(),
                            },
                            GuiPresetSource {
                                name: Some("test-preset-2".to_string()),
                                value: "0.3".to_string(),
                            },
                        ],
                    },
                    GuiFieldDefinitionSource {
                        binding: "test-binding-2".to_string(),
                        name: "test-name-2".to_string(),
                        description: "test-description-2".to_string(),
                        presets: vec![
                            GuiPresetSource {
                                name: None,
                                value: "3.2".to_string(),
                            },
                            GuiPresetSource {
                                name: None,
                                value: "4.8".to_string(),
                            },
                        ],
                    },
                    GuiFieldDefinitionSource {
                        binding: "test-binding-3".to_string(),
                        name: "test-name-3".to_string(),
                        description: "test-description-3".to_string(),
                        presets: vec![
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
                        ],
                    },
                ],
            }],
        };
        let scenario = Scenario {
            name: "scenario1".into(),
            bindings: HashMap::new(),
            deployer: mock_deployer(),
            runs: None,
            blocks: None,
        };
        let order = Order {
            inputs: vec![],
            outputs: vec![],
            network: mock_network(),
            deployer: None,
            orderbook: None,
        };
        let deployment = Deployment {
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
        let deployment = &gui.deployments[0];
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
        assert_eq!(field1.description, "test-description");
        assert_eq!(field1.presets.len(), 2);
        assert_eq!(field1.presets[0].name, Some("test-preset".to_string()));
        assert_eq!(field1.presets[0].value, "0.015".to_string());
        assert_eq!(field1.presets[1].name, Some("test-preset-2".to_string()));
        assert_eq!(field1.presets[1].value, "0.3".to_string());
        let field2 = &deployment.fields[1];
        assert_eq!(field2.binding, "test-binding-2");
        assert_eq!(field2.name, "test-name-2");
        assert_eq!(field2.description, "test-description-2");
        assert_eq!(field2.presets.len(), 2);
        assert_eq!(field2.presets[0].name, None);
        assert_eq!(field2.presets[1].name, None);
        assert_eq!(field2.presets[1].value, "4.8".to_string());
        let field3 = &deployment.fields[2];
        assert_eq!(field3.binding, "test-binding-3");
        assert_eq!(field3.name, "test-name-3");
        assert_eq!(field3.description, "test-description-3");
        assert_eq!(field3.presets.len(), 3);
        assert_eq!(field3.presets[0].value, Address::default().to_string());
        assert_eq!(field3.presets[1].value, "some-value".to_string());
        assert_eq!(field3.presets[2].value, "true".to_string());
    }
}
