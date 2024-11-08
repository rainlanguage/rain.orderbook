use alloy::primitives::{
    ruint::ParseError,
    utils::{parse_units, UnitsError},
    Address, U256,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

use crate::{Deployment, DeploymentRef, Token, TokenRef};

pub const GUI_PRESET_VALUE_DECIMALS: u8 = 18;

// Config source for Gui

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum GuiFieldValueSource {
    Text(String),
    Number(f64),
    Address(Address),
    Boolean(bool),
}
impl GuiFieldValueSource {
    fn try_into_gui_field_value(self) -> Result<GuiFieldValue, ParseGuiConfigSourceError> {
        match self {
            GuiFieldValueSource::Text(text) => Ok(GuiFieldValue::Text(text)),
            GuiFieldValueSource::Number(number) => Ok(GuiFieldValue::Number(
                parse_units(&number.to_string(), GUI_PRESET_VALUE_DECIMALS)?.into(),
            )),
            GuiFieldValueSource::Address(address) => Ok(GuiFieldValue::Address(address)),
            GuiFieldValueSource::Boolean(boolean) => Ok(GuiFieldValue::Boolean(boolean)),
        }
    }
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiPresetSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: GuiFieldValueSource,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiDepositSource {
    pub token: TokenRef,
    #[typeshare(typescript(type = "number"))]
    pub min: Option<f64>,
    pub presets: Vec<f64>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiFieldDefinitionSource {
    pub binding: String,
    pub name: String,
    pub description: String,
    #[typeshare(typescript(type = "number"))]
    pub min: Option<f64>,
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

                        let presets = deposit_source
                            .presets
                            .iter()
                            .map(|preset| {
                                let amount = parse_units(
                                    &preset.to_string(),
                                    token.decimals.unwrap_or(GUI_PRESET_VALUE_DECIMALS),
                                )?;
                                Ok(amount.into())
                            })
                            .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                        Ok(GuiDeposit {
                            token: token.clone(),
                            min: deposit_source
                                .min
                                .map(|min| {
                                    parse_units(
                                        &min.to_string(),
                                        token.decimals.unwrap_or(GUI_PRESET_VALUE_DECIMALS),
                                    )
                                })
                                .transpose()?
                                .map(Into::into),
                            presets,
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
                            min: field_source
                                .min
                                .map(|min| {
                                    parse_units(&min.to_string(), GUI_PRESET_VALUE_DECIMALS)
                                        .map(Into::into)
                                        .map_err(ParseGuiConfigSourceError::from)
                                })
                                .transpose()?,
                            presets: field_source
                                .presets
                                .iter()
                                .map(|preset| {
                                    Ok(GuiPreset {
                                        name: preset.name.clone(),
                                        value: preset.value.clone().try_into_gui_field_value()?,
                                    })
                                })
                                .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                Ok(GuiDeployment {
                    deployment,
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
pub enum GuiFieldValue {
    Text(String),
    Number(U256),
    Address(Address),
    Boolean(bool),
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GuiPreset {
    name: Option<String>,
    value: GuiFieldValue,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GuiDeposit {
    #[typeshare(typescript(type = "Token"))]
    token: Arc<Token>,
    min: Option<U256>,
    presets: Vec<U256>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GuiDeployment {
    #[typeshare(typescript(type = "Deployment"))]
    deployment: Arc<Deployment>,
    name: String,
    description: String,
    deposits: Vec<GuiDeposit>,
    fields: Vec<GuiFieldDefinition>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GuiFieldDefinition {
    binding: String,
    name: String,
    description: String,
    #[typeshare(typescript(type = "string"))]
    min: Option<U256>,
    presets: Vec<GuiPreset>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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
                    min: Some(1.0),
                    presets: vec![1.3, 2.7],
                }],
                fields: vec![
                    GuiFieldDefinitionSource {
                        binding: "test-binding".to_string(),
                        name: "test-name".to_string(),
                        description: "test-description".to_string(),
                        min: None,
                        presets: vec![
                            GuiPresetSource {
                                name: Some("test-preset".to_string()),
                                value: GuiFieldValueSource::Number(0.015),
                            },
                            GuiPresetSource {
                                name: Some("test-preset-2".to_string()),
                                value: GuiFieldValueSource::Number(0.3),
                            },
                        ],
                    },
                    GuiFieldDefinitionSource {
                        binding: "test-binding-2".to_string(),
                        name: "test-name-2".to_string(),
                        description: "test-description-2".to_string(),
                        min: Some(2.0),
                        presets: vec![
                            GuiPresetSource {
                                name: None,
                                value: GuiFieldValueSource::Number(3.2),
                            },
                            GuiPresetSource {
                                name: None,
                                value: GuiFieldValueSource::Number(4.8),
                            },
                        ],
                    },
                    GuiFieldDefinitionSource {
                        binding: "test-binding-3".to_string(),
                        name: "test-name-3".to_string(),
                        description: "test-description-3".to_string(),
                        min: None,
                        presets: vec![
                            GuiPresetSource {
                                name: None,
                                value: GuiFieldValueSource::Address(Address::default()),
                            },
                            GuiPresetSource {
                                name: None,
                                value: GuiFieldValueSource::Text("some-value".to_string()),
                            },
                            GuiPresetSource {
                                name: None,
                                value: GuiFieldValueSource::Boolean(true),
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
        assert_eq!(deposit.min, Some(U256::from(1000000000000000000_u64)));
        assert_eq!(deposit.presets.len(), 2);
        assert_eq!(deposit.presets[0], U256::from(1300000000000000000_u64));
        assert_eq!(deposit.presets[1], U256::from(2700000000000000000_u64));
        assert_eq!(deployment.fields.len(), 3);
        let field1 = &deployment.fields[0];
        assert_eq!(field1.binding, "test-binding");
        assert_eq!(field1.name, "test-name");
        assert_eq!(field1.description, "test-description");
        assert_eq!(field1.min, None);
        assert_eq!(field1.presets.len(), 2);
        assert_eq!(field1.presets[0].name, Some("test-preset".to_string()));
        assert_eq!(
            field1.presets[0].value,
            GuiFieldValue::Number(U256::from(15000000000000000_u64))
        );
        assert_eq!(field1.presets[1].name, Some("test-preset-2".to_string()));
        assert_eq!(
            field1.presets[1].value,
            GuiFieldValue::Number(U256::from(300000000000000000_u64))
        );
        let field2 = &deployment.fields[1];
        assert_eq!(field2.binding, "test-binding-2");
        assert_eq!(field2.name, "test-name-2");
        assert_eq!(field2.description, "test-description-2");
        assert_eq!(field2.min, Some(U256::from(2000000000000000000_u64)));
        assert_eq!(field2.presets.len(), 2);
        assert_eq!(field2.presets[0].name, None);
        assert_eq!(
            field2.presets[0].value,
            GuiFieldValue::Number(U256::from(3200000000000000000_u64))
        );
        assert_eq!(field2.presets[1].name, None);
        assert_eq!(
            field2.presets[1].value,
            GuiFieldValue::Number(U256::from(4800000000000000000_u64))
        );
        let field3 = &deployment.fields[2];
        assert_eq!(field3.binding, "test-binding-3");
        assert_eq!(field3.name, "test-name-3");
        assert_eq!(field3.description, "test-description-3");
        assert_eq!(field3.min, None);
        assert_eq!(field3.presets.len(), 3);
        assert_eq!(
            field3.presets[0].value,
            GuiFieldValue::Address(Address::default())
        );
        assert_eq!(
            field3.presets[1].value,
            GuiFieldValue::Text("some-value".to_string())
        );
        assert_eq!(field3.presets[2].value, GuiFieldValue::Boolean(true));
    }
}
