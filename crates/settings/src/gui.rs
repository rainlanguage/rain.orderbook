use crate::{
    network_bindings::NetworkBinding,
    yaml::{
        context::{Context, GuiContextTrait},
        default_document, get_hash_value, optional_hash, optional_string, optional_vec,
        require_string, require_vec, FieldErrorKind, YamlError, YamlParsableHash,
        YamlParseableValue,
    },
    Deployment, Network, Token, TokenRef,
};
use alloy::primitives::{ruint::ParseError, utils::UnitsError};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
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
    pub presets: Option<Vec<String>>,
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
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SelectNetwork {
    pub name: String,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(SelectNetwork);

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GuiConfigSource {
    pub name: String,
    pub description: String,
    pub deployments: HashMap<String, GuiDeploymentSource>,
    pub select_networks: Option<HashMap<String, SelectNetwork>>,
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
                            token: Some(token.clone()),
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
                        document: default_document(),
                        key: deployment_name.to_string(),
                        deployment,
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
            select_networks: self.select_networks,
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
    #[typeshare(typescript(type = "Token | undefined"))]
    pub token: Option<Arc<Token>>,
    pub presets: Option<Vec<String>>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(GuiDeposit);

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiDeployment {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "Deployment"))]
    pub deployment: Arc<Deployment>,
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDeposit>,
    pub fields: Vec<GuiFieldDefinition>,
    pub select_tokens: Option<Vec<String>>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(GuiDeployment);

impl PartialEq for GuiDeployment {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.deployment == other.deployment
            && self.name == other.name
            && self.description == other.description
            && self.deposits == other.deposits
            && self.fields == other.fields
            && self.select_tokens == other.select_tokens
    }
}

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
    pub select_networks: Option<HashMap<String, SelectNetwork>>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Gui);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct NameAndDescription {
    pub name: String,
    pub description: String,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(NameAndDescription);

impl Gui {
    pub fn parse_deployment_keys(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<Vec<String>, YamlError> {
        let mut deployment_keys = Vec::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let deployments = gui
                    .get(&StrictYaml::String("deployments".to_string()))
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    })?;

                if let StrictYaml::Hash(deployments_hash) = deployments {
                    for (key, _) in deployments_hash {
                        if let StrictYaml::String(key) = key {
                            deployment_keys.push(key.clone());
                        }
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    });
                }
            }
        }

        Ok(deployment_keys)
    }

    pub fn parse_select_tokens(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployment_key: &str,
    ) -> Result<Option<Vec<String>>, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                if let Some(StrictYaml::Hash(deployments_hash)) =
                    gui.get(&StrictYaml::String("deployments".to_string()))
                {
                    if let Some(StrictYaml::Hash(deployment_hash)) =
                        deployments_hash.get(&StrictYaml::String(deployment_key.to_string()))
                    {
                        if let Some(StrictYaml::Array(tokens)) =
                            deployment_hash.get(&StrictYaml::String("select-tokens".to_string()))
                        {
                            let mut result = Vec::new();
                            for token in tokens {
                                if let StrictYaml::String(token_str) = token {
                                    result.push(token_str.clone());
                                }
                            }
                            return Ok(Some(result));
                        }
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    });
                }
            }
        }
        Ok(None)
    }

    pub fn parse_strategy_details(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<NameAndDescription, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let name = require_string(
                    get_hash_value(gui, "name", Some("gui".to_string()))?,
                    None,
                    Some("gui".to_string()),
                )?;

                let description = require_string(
                    get_hash_value(gui, "description", Some("gui".to_string()))?,
                    None,
                    Some("gui".to_string()),
                )?;

                return Ok(NameAndDescription { name, description });
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing("name/description".to_string()),
            location: "gui".to_string(),
        })
    }

    pub fn parse_deployment_details(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, NameAndDescription>, YamlError> {
        let mut deployment_details = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let deployments = gui
                    .get(&StrictYaml::String("deployments".to_string()))
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    })?
                    .as_hash()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                for (key_yaml, deployment_yaml) in deployments {
                    let deployment_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("gui deployment '{deployment_key}'");

                    let name =
                        require_string(deployment_yaml, Some("name"), Some(location.clone()))?;

                    let description = require_string(
                        deployment_yaml,
                        Some("description"),
                        Some(location.clone()),
                    )?;

                    deployment_details
                        .insert(deployment_key, NameAndDescription { name, description });
                }
            }
        }

        Ok(deployment_details)
    }

    pub fn parse_field_presets(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployment_key: &str,
        field_binding: &str,
    ) -> Result<Option<Vec<GuiPreset>>, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                if let Some(StrictYaml::Hash(deployments_hash)) =
                    gui.get(&StrictYaml::String("deployments".to_string()))
                {
                    if let Some(StrictYaml::Hash(deployment_hash)) =
                        deployments_hash.get(&StrictYaml::String(deployment_key.to_string()))
                    {
                        if let Some(StrictYaml::Array(fields)) =
                            deployment_hash.get(&StrictYaml::String("fields".to_string()))
                        {
                            for (field_index, field) in fields.iter().enumerate() {
                                if let StrictYaml::Hash(field_hash) = field {
                                    if let Some(StrictYaml::String(binding)) =
                                        field_hash.get(&StrictYaml::String("binding".to_string()))
                                    {
                                        if binding == field_binding {
                                            return match optional_vec(field, "presets") {
                                                Some(presets) => {
                                                    let preset_vec = presets.iter().enumerate()
                                                        .map(|(preset_index, preset_yaml)| {
                                                            let name = optional_string(preset_yaml, "name");
                                                            let value = require_string(
                                                                preset_yaml,
                                                                Some("value"),
                                                                Some(format!(
                                                                    "preset index '{preset_index}' for field index '{field_index}' in gui deployment '{deployment_key}'",
                                                                ))
                                                            )?;

                                                            Ok(GuiPreset {
                                                                id: preset_index.to_string(),
                                                                name,
                                                                value,
                                                            })
                                                        })
                                                        .collect::<Result<Vec<_>, YamlError>>()?;
                                                    Ok(Some(preset_vec))
                                                }
                                                None => Ok(None),
                                            };
                                        }
                                    } else {
                                        return Err(YamlError::Field {
                                            kind: FieldErrorKind::Missing("binding".to_string()),
                                            location: format!("field index: {field_index} in gui deployment '{deployment_key}'"),
                                        });
                                    }
                                }
                            }
                        } else {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::Missing("fields".to_string()),
                                location: format!("gui deployment '{deployment_key}'"),
                            });
                        }
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    });
                }
            }
        }
        Ok(None)
    }

    pub fn parse_select_networks(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<Option<HashMap<String, SelectNetwork>>, YamlError> {
        let network_keys = Network::parse_network_keys(documents.clone())?;

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                if let Some(StrictYaml::Hash(select_networks_hash)) =
                    gui.get(&StrictYaml::String("select-networks".to_string()))
                {
                    let mut result = HashMap::new();

                    for (network_key, network_yaml) in select_networks_hash {
                        let network_key = network_key.as_str().unwrap_or_default().to_string();
                        let location = format!("select-networks '{network_key}'");

                        if !network_keys.contains(&network_key) {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "networks".to_string(),
                                    reason: format!("Network '{}' not found", network_key),
                                },
                                location: location.clone(),
                            });
                        }

                        let name =
                            require_string(network_yaml, Some("name"), Some(location.clone()))?;

                        result.insert(network_key, SelectNetwork { name });
                    }

                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }
}

impl YamlParseableValue for Gui {
    fn parse_from_yaml(
        _: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<Self, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }

    fn parse_from_yaml_optional(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<Option<Self>, YamlError> {
        let mut gui_res: Option<Gui> = None;
        let mut gui_deployments_res: HashMap<String, GuiDeployment> = HashMap::new();

        let tokens = Token::parse_all_from_yaml(documents.clone(), None);

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let name = get_hash_value(gui, "name", Some("gui".to_string()))?
                    .as_str()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "name".to_string(),
                            expected: "a string".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                let description = get_hash_value(gui, "description", Some("gui".to_string()))?
                    .as_str()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "description".to_string(),
                            expected: "a string".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                let select_networks = Gui::parse_select_networks(documents.clone())?;

                if gui_res.is_none() {
                    gui_res = Some(Gui {
                        name: name.to_string(),
                        description: description.to_string(),
                        deployments: gui_deployments_res.clone(),
                        select_networks,
                    });
                }

                let deployments = gui
                    .get(&StrictYaml::String("deployments".to_string()))
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    })?
                    .as_hash()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                for (deployment_name, deployment_yaml) in deployments {
                    let deployment_name = deployment_name.as_str().unwrap_or_default().to_string();
                    let location = format!("gui deployment '{deployment_name}'");

                    if let Some(context) = context {
                        if let Some(current_deployment) = context.get_current_deployment() {
                            if current_deployment != &deployment_name {
                                continue;
                            }
                        }
                    }

                    let mut context = Context::from_context(context);

                    let network_bindings =
                        NetworkBinding::parse_all_from_yaml(documents.clone(), Some(&context))?;
                    context.add_network_bindings(
                        network_bindings
                            .iter()
                            .map(|(k, v)| (k.clone(), Arc::new(v.clone())))
                            .collect(),
                    );

                    let select_tokens = match optional_vec(deployment_yaml, "select-tokens") {
                            Some(tokens) => Some(
                                tokens
                                    .iter()
                                    .enumerate()
                                    .map(|(select_token_index, select_token_value)| {
                                        Ok(select_token_value.as_str().ok_or(YamlError::Field {
                                            kind: FieldErrorKind::InvalidType {
                                                field: "select-token".to_string(),
                                                expected: "a string".to_string(),
                                            },
                                            location: format!("select-token index '{select_token_index}' in {location}"),
                                        })?.to_string())
                                    })
                                    .collect::<Result<Vec<_>, YamlError>>()?,
                            ),
                            None => None,
                        };
                    if let Some(ref select_tokens) = select_tokens {
                        context.add_select_tokens(select_tokens.clone());
                    }

                    let deployment = Deployment::parse_from_yaml(
                        documents.clone(),
                        &deployment_name,
                        Some(&context),
                    )?;
                    context.add_order(deployment.order.clone());

                    let name =
                        require_string(deployment_yaml, Some("name"), Some(location.clone()))?;

                    let description = require_string(
                        deployment_yaml,
                        Some("description"),
                        Some(location.clone()),
                    )?;

                    let deposits = require_vec(
                        deployment_yaml,
                        "deposits",
                        Some(location.clone()),
                    )?.iter().enumerate().map(|(deposit_index, deposit_value)| {
                        let mut deposit_token = None;

                        if let Ok(tokens) = &tokens {
                            let token = tokens.get(&require_string(
                                deposit_value,
                                Some("token"),
                                Some(format!(
                                    "deposit index '{deposit_index}' in {location}",
                                )),
                            )?);

                            deposit_token = token.map(|token| Arc::new(token.clone()));
                        }

                        let presets = match optional_vec(deposit_value, "presets") {
                            Some(presets) => Some(presets.iter()
                            .enumerate()
                            .map(|(preset_index, preset_yaml)| {
                                Ok(preset_yaml.as_str().ok_or(YamlError::Field{
                                    kind: FieldErrorKind::InvalidType {
                                        field: "preset value".to_string(),
                                        expected: "a string".to_string(),
                                    },
                                    location: format!(
                                        "presets list index '{preset_index}' for deposit index '{deposit_index}' in {location}",
                                    ),
                                })?.to_string())
                            })
                            .collect::<Result<Vec<_>, YamlError>>()?),
                            None => None,
                        };

                        let gui_deposit = GuiDeposit {
                            token: deposit_token,
                            presets,
                        };
                        Ok(gui_deposit)
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let fields = require_vec(
                        deployment_yaml,
                        "fields",
                        Some(location.clone()),
                    )?.iter().enumerate().map(|(field_index, field_yaml)| {
                        let binding = require_string(
                            field_yaml,
                            Some("binding"),
                            Some(format!(
                                "fields list index '{field_index}' in {location}",
                            )),
                        )?;

                        let name = require_string(
                            field_yaml,
                            Some("name"),
                            Some(format!(
                                "fields list index '{field_index}' in {location}",
                            )),
                        )?;
                        let interpolated_name = context.interpolate(&name)?;

                        let description = optional_string(field_yaml, "description");
                        let interpolated_description = description.map(|description| context.interpolate(&description)).transpose()?;

                        let presets = match optional_vec(field_yaml, "presets") {
                            Some(p) => Some(p.iter().enumerate().map(|(preset_index, preset_yaml)| {
                                let name = optional_string(preset_yaml, "name");
                                let value = require_string(
                                    preset_yaml,
                                    Some("value"),
                                    Some(format!(
                                        "preset index '{preset_index}' for field index '{field_index}' in {location}",
                                    ))
                                )?;

                                let gui_preset = GuiPreset {
                                    id: preset_index.to_string(),
                                    name,
                                    value,
                                };
                                Ok(gui_preset)
                            })
                            .collect::<Result<Vec<_>, YamlError>>()?),
                            None => None,
                        };

                        let gui_field_definition = GuiFieldDefinition {
                            binding,
                            name: interpolated_name,
                            description: interpolated_description,
                            presets
                        };
                        Ok(gui_field_definition)
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let gui_deployment = GuiDeployment {
                        document: document.clone(),
                        key: deployment_name.clone(),
                        deployment: Arc::new(deployment),
                        name,
                        description,
                        deposits,
                        fields,
                        select_tokens,
                    };

                    if gui_deployments_res.contains_key(&deployment_name) {
                        return Err(YamlError::KeyShadowing(deployment_name));
                    }
                    gui_deployments_res.insert(deployment_name, gui_deployment);
                }
                if let Some(gui) = &mut gui_res {
                    gui.deployments.clone_from(&gui_deployments_res);
                }
            }
        }

        Ok(gui_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{mock_deployer, mock_network, mock_token},
        yaml::tests::get_document,
        Order, Scenario,
    };
    use alloy::primitives::Address;
    use std::sync::RwLock;
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
                        presets: Some(vec!["1.3".to_string(), "2.7".to_string()]),
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
            select_networks: None,
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
        assert_eq!(
            deposit.token.as_ref().unwrap().label,
            Some("test-token".to_string())
        );
        let presets = deposit.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0], "1.3".to_string());
        assert_eq!(presets[1], "2.7".to_string());
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

    #[test]
    fn test_parse_gui_from_yaml() {
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    test: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name:
      - test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "name".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name:
      - test: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "name".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description:
      - test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "description".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description:
      - test: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "description".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
        - test: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
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
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        deployer: deployer1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            test: test
"#;
        let error = Gui::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "root".to_string(),
            }
        );

        let yaml_prefix = r#"
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
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        deployer: deployer1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            test: test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: deployment1
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: some name
            description: some description
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deposits".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - test: test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("token".to_string()),
                location: "deposit index '0' in gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - test: test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "preset value".to_string(),
                    expected: "a string".to_string()
                },
                location:
                    "presets list index '0' for deposit index '0' in gui deployment 'deployment1'"
                        .to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("fields".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - test: test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("binding".to_string()),
                location: "fields list index '0' in gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "fields list index '0' in gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value:
                        - test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "value".to_string(),
                    expected: "a string".to_string()
                },
                location: "preset index '0' for field index '0' in gui deployment 'deployment1'"
                    .to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
            select-tokens:
                - test: test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "select-token".to_string(),
                    expected: "a string".to_string()
                },
                location: "select-token index '0' in gui deployment 'deployment1'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_gui_from_yaml_multiple_files() {
        let yaml_one = r#"
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
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        deployer: deployer1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
    deployment2:
        scenario: scenario1
        order: order1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let yaml_two = r#"
gui:
    name: test
    description: test
    deployments:
        deployment2:
            name: test another
            description: test another
            deposits:
                - token: token2
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let res = Gui::parse_from_yaml_optional(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        let gui = res.unwrap();
        assert_eq!(gui.deployments.len(), 2);

        let deployment = gui.deployments.get("deployment1").unwrap();
        assert_eq!(deployment.name, "test");
        assert_eq!(deployment.description, "test");
        assert_eq!(deployment.deposits[0].token.as_ref().unwrap().key, "token1");

        let deployment = gui.deployments.get("deployment2").unwrap();
        assert_eq!(deployment.name, "test another");
        assert_eq!(deployment.description, "test another");
        assert_eq!(deployment.deposits[0].token.as_ref().unwrap().key, "token2");
    }

    #[test]
    fn test_parse_gui_from_yaml_duplicate_key() {
        let yaml_one = r#"
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
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        deployer: deployer1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
    deployment2:
        scenario: scenario1
        order: order1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let yaml_two = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let error = Gui::parse_from_yaml_optional(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();

        assert_eq!(error, YamlError::KeyShadowing("deployment1".to_string()));
    }

    #[test]
    fn test_parse_deployment_keys() {
        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
"#;

        let error = Gui::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments: test
"#;

        let error = Gui::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
      - test
"#;

        let error = Gui::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
      - test: test
"#;

        let error = Gui::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
      test: test
      test2: test2
"#;

        let keys = Gui::parse_deployment_keys(vec![get_document(yaml)]).unwrap();
        assert_eq!(keys, vec!["test".to_string(), "test2".to_string()]);
    }

    #[test]
    fn test_parse_select_networks() {
        let yaml = r#"
gui:
    select-networks:
        network1:
            name: Network One
        network2:
            name: Network Two
"#;
        let error = Gui::parse_select_networks(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "networks".to_string(),
                    reason: "Network 'network1' not found".to_string(),
                },
                location: "select-networks 'network1'".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
    network2:
        rpc: https://eth.llamarpc.com
        chain-id: 1
gui:
    select-networks:
        network1:
            name: Network One
        network2:
            name: Network Two
"#;
        let networks = Gui::parse_select_networks(vec![get_document(yaml)])
            .unwrap()
            .unwrap();
        assert_eq!(networks.len(), 2);
        assert_eq!(networks.get("network1").unwrap().name, "Network One");
        assert_eq!(networks.get("network2").unwrap().name, "Network Two");

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
gui:
    select-networks:
        network1:
            name:
                - invalid
"#;
        let error = Gui::parse_select_networks(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "name".to_string(),
                    expected: "a string".to_string(),
                },
                location: "select-networks 'network1'".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
gui:
    select-networks:
        network1:
            name:
                - invalid: invalid
"#;
        let error = Gui::parse_select_networks(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "name".to_string(),
                    expected: "a string".to_string(),
                },
                location: "select-networks 'network1'".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpc: https://eth.llamarpc.com
        chain-id: 1
gui:
    select-networks:
        network1:
            wrong: value
"#;
        let error = Gui::parse_select_networks(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "select-networks 'network1'".to_string(),
            }
        );
    }
}
