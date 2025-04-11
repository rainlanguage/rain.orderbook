use crate::{
    yaml::{
        context::{Context, GuiContextTrait},
        default_document, get_hash_value, get_hash_value_as_option, optional_hash, optional_string,
        optional_vec, require_string, require_vec, FieldErrorKind, YamlError, YamlParsableHash,
        YamlParseableValue,
    },
    DeploymentCfg, TokenCfg, TokenCfgRef,
};
use alloy::primitives::{ruint::ParseError, utils::UnitsError};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, serialize_hashmap_as_object};

// Config source for Gui
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiPresetSourceCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiPresetSourceCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiDepositSourceCfg {
    pub token: TokenCfgRef,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiFieldDefinitionSourceCfg {
    pub binding: String,
    pub name: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub description: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<GuiPresetSourceCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub default: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub show_custom_field: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiDeploymentSourceCfg {
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDepositSourceCfg>,
    pub fields: Vec<GuiFieldDefinitionSourceCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_tokens: Option<Vec<GuiSelectTokensCfg>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiConfigSourceCfg {
    pub name: String,
    pub description: String,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, GuiDeploymentSourceCfg>")
    )]
    pub deployments: HashMap<String, GuiDeploymentSourceCfg>,
}
impl GuiConfigSourceCfg {
    pub fn try_into_gui(
        self,
        deployments: &HashMap<String, Arc<DeploymentCfg>>,
        tokens: &HashMap<String, Arc<TokenCfg>>,
    ) -> Result<GuiCfg, ParseGuiConfigSourceError> {
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

                        Ok(GuiDepositCfg {
                            token: Some(token.clone()),
                            presets: deposit_source.presets.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                let fields = deployment_source
                    .fields
                    .iter()
                    .map(|field_source| {
                        Ok(GuiFieldDefinitionCfg {
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
                                            Ok(GuiPresetCfg {
                                                id: i.to_string(),
                                                name: preset.name.clone(),
                                                value: preset.value.clone(),
                                            })
                                        })
                                        .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()
                                })
                                .transpose()?,
                            default: field_source.default.clone(),
                            show_custom_field: field_source.show_custom_field,
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                Ok((
                    deployment_name.clone(),
                    GuiDeploymentCfg {
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

        Ok(GuiCfg {
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiPresetCfg {
    pub id: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub name: Option<String>,
    pub value: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiPresetCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiDepositCfg {
    pub token: Option<Arc<TokenCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<String>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiDepositCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiSelectTokensCfg {
    pub key: TokenCfgRef,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub name: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiDeploymentCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub deployment: Arc<DeploymentCfg>,
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDepositCfg>,
    pub fields: Vec<GuiFieldDefinitionCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub select_tokens: Option<Vec<GuiSelectTokensCfg>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiDeploymentCfg);

impl PartialEq for GuiDeploymentCfg {
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct GuiFieldDefinitionCfg {
    pub binding: String,
    pub name: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub description: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<GuiPresetCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub default: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub show_custom_field: Option<bool>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiFieldDefinitionCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiCfg {
    pub name: String,
    pub description: String,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, GuiDeploymentCfg>")
    )]
    pub deployments: HashMap<String, GuiDeploymentCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiCfg);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct NameAndDescriptionCfg {
    pub name: String,
    pub description: String,
    pub short_description: Option<String>,
}

#[cfg(target_family = "wasm")]
impl_wasm_traits!(NameAndDescriptionCfg);

impl GuiCfg {
    pub fn check_gui_key_exists(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<bool, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
            if let Some(gui) = optional_hash(&document_read, "gui") {
                return Ok(true);
            }
        }
        Ok(false)
    }

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
    ) -> Result<Option<Vec<GuiSelectTokensCfg>>, YamlError> {
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
                            for (index, token) in tokens.iter().enumerate() {
                                if let StrictYaml::Hash(token_hash) = token {
                                    let key = get_hash_value(token_hash, "key", Some(format!("key string missing for select-token index: {index} in gui deployment: {deployment_key}")))?.as_str().ok_or(YamlError::Field {
                                        kind: FieldErrorKind::Missing("key".to_string()),
                                        location: format!("select-token index: {index} in gui deployment: {deployment_key}"),
                                    })?;
                                    let name = get_hash_value_as_option(token_hash, "name")
                                        .map(|s| s.as_str())
                                        .unwrap_or_default();
                                    let description =
                                        get_hash_value_as_option(token_hash, "description")
                                            .map(|s| s.as_str())
                                            .unwrap_or_default();
                                    result.push(GuiSelectTokensCfg {
                                        key: key.to_string(),
                                        name: name.map(|s| s.to_string()),
                                        description: description.map(|s| s.to_string()),
                                    });
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
    ) -> Result<NameAndDescriptionCfg, YamlError> {
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

                let short_description = require_string(
                    get_hash_value(gui, "short-description", Some("gui".to_string()))?,
                    None,
                    Some("gui".to_string()),
                )?;

                return Ok(NameAndDescriptionCfg {
                    name,
                    description,
                    short_description: Some(short_description),
                });
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing("name/description".to_string()),
            location: "gui".to_string(),
        })
    }

    pub fn parse_deployment_details(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, NameAndDescriptionCfg>, YamlError> {
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

                    let short_description = optional_string(deployment_yaml, "short-description");

                    deployment_details.insert(
                        deployment_key,
                        NameAndDescriptionCfg {
                            name,
                            description,
                            short_description,
                        },
                    );
                }
            }
        }

        Ok(deployment_details)
    }

    pub fn parse_field_presets(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployment_key: &str,
        field_binding: &str,
    ) -> Result<Option<Vec<GuiPresetCfg>>, YamlError> {
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

                                                            Ok(GuiPresetCfg {
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
}

impl YamlParseableValue for GuiCfg {
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
        let mut gui_res: Option<GuiCfg> = None;
        let mut gui_deployments_res: HashMap<String, GuiDeploymentCfg> = HashMap::new();

        let tokens = TokenCfg::parse_all_from_yaml(documents.clone(), context);

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

                if gui_res.is_none() {
                    gui_res = Some(GuiCfg {
                        name: name.to_string(),
                        description: description.to_string(),
                        deployments: gui_deployments_res.clone(),
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

                    let select_tokens = match optional_vec(deployment_yaml, "select-tokens") {
                            Some(tokens) => Some(
                                tokens
                                    .iter()
                                    .enumerate()
                                    .map(|(select_token_index, select_token_value)| {
                                        let location = format!("select-token index '{select_token_index}' in gui deployment '{deployment_name}'");

                                        select_token_value.as_hash().ok_or(YamlError::Field{
                                            kind: FieldErrorKind::InvalidType {
                                                field: "select-token".to_string(),
                                                expected: "a map".to_string(),
                                            },
                                            location: location.clone(),
                                        })?;

                                        Ok(GuiSelectTokensCfg {
                                            key: require_string(select_token_value, Some("key"), Some(location.clone()))?,
                                            name: optional_string(select_token_value, "name"),
                                            description: optional_string(select_token_value, "description"),
                                    })
                                })
                                .collect::<Result<Vec<_>, YamlError>>()?,
                        ),
                        None => None,
                    };
                    if let Some(ref select_tokens) = select_tokens {
                        context.add_select_tokens(
                            select_tokens
                                .iter()
                                .map(|select_token| select_token.key.clone())
                                .collect::<Vec<_>>(),
                        );
                    }

                    let deployment = DeploymentCfg::parse_from_yaml(
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

                        let gui_deposit = GuiDepositCfg {
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

                                let gui_preset = GuiPresetCfg {
                                    id: preset_index.to_string(),
                                    name,
                                    value,
                                };
                                Ok(gui_preset)
                            })
                            .collect::<Result<Vec<_>, YamlError>>()?),
                            None => None,
                        };

                        let default = optional_string(field_yaml, "default");
                        let show_custom_field = optional_string(field_yaml, "show-custom-field").map(|v| v.eq("true"));

                        let gui_field_definition = GuiFieldDefinitionCfg {
                            binding,
                            name: interpolated_name,
                            description: interpolated_description,
                            presets,
                            default,
                            show_custom_field
                        };
                        Ok(gui_field_definition)
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let gui_deployment = GuiDeploymentCfg {
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
                        return Err(YamlError::KeyShadowing(
                            deployment_name.clone(),
                            "gui deployment".to_string(),
                        ));
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
        OrderCfg, ScenarioCfg,
    };
    use alloy::primitives::Address;
    use std::sync::RwLock;
    use strict_yaml_rust::StrictYaml;

    #[test]
    fn test_gui_creation_success() {
        let gui_config_source = GuiConfigSourceCfg {
            name: "test-gui".to_string(),
            description: "test-gui-description".to_string(),
            deployments: HashMap::from([(
                "test-deployment".to_string(),
                GuiDeploymentSourceCfg {
                    name: "test-deployment".to_string(),
                    description: "test-deployment-description".to_string(),
                    deposits: vec![GuiDepositSourceCfg {
                        token: "test-token".to_string(),
                        presets: Some(vec!["1.3".to_string(), "2.7".to_string()]),
                    }],
                    fields: vec![
                        GuiFieldDefinitionSourceCfg {
                            binding: "test-binding".to_string(),
                            name: "test-name".to_string(),
                            description: Some("test-description".to_string()),
                            presets: Some(vec![
                                GuiPresetSourceCfg {
                                    name: Some("test-preset".to_string()),
                                    value: "0.015".to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: Some("test-preset-2".to_string()),
                                    value: "0.3".to_string(),
                                },
                            ]),
                            default: None,
                            show_custom_field: None,
                        },
                        GuiFieldDefinitionSourceCfg {
                            binding: "test-binding-2".to_string(),
                            name: "test-name-2".to_string(),
                            description: Some("test-description-2".to_string()),
                            presets: Some(vec![
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "3.2".to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "4.8".to_string(),
                                },
                            ]),
                            default: Some("0.015".to_string()),
                            show_custom_field: Some(true),
                        },
                        GuiFieldDefinitionSourceCfg {
                            binding: "test-binding-3".to_string(),
                            name: "test-name-3".to_string(),
                            description: Some("test-description-3".to_string()),
                            presets: Some(vec![
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: Address::default().to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "some-value".to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "true".to_string(),
                                },
                            ]),
                            default: Some("0.25".to_string()),
                            show_custom_field: Some(false),
                        },
                    ],
                    select_tokens: Some(vec![GuiSelectTokensCfg {
                        key: "test-token".to_string(),
                        name: Some("Test name".to_string()),
                        description: Some("Test description".to_string()),
                    }]),
                },
            )]),
        };
        let scenario = ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "scenario1".into(),
            bindings: HashMap::new(),
            deployer: mock_deployer(),
            runs: None,
            blocks: None,
        };
        let order = OrderCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            inputs: vec![],
            outputs: vec![],
            network: mock_network(),
            deployer: None,
            orderbook: None,
        };
        let deployment = DeploymentCfg {
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
        assert_eq!(field1.default, None);
        assert_eq!(field1.show_custom_field, None);
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
        assert_eq!(field2.default, Some("0.015".to_string()));
        assert_eq!(field2.show_custom_field, Some(true));
        let presets = field2.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, None);
        assert_eq!(presets[1].name, None);
        assert_eq!(presets[1].value, "4.8".to_string());
        let field3 = &deployment.fields[2];
        assert_eq!(field3.binding, "test-binding-3");
        assert_eq!(field3.name, "test-name-3");
        assert_eq!(field3.description, Some("test-description-3".to_string()));
        assert_eq!(field3.default, Some("0.25".to_string()));
        assert_eq!(field3.show_custom_field, Some(false));
        let presets = field3.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].value, Address::default().to_string());
        assert_eq!(presets[1].value, "some-value".to_string());
        assert_eq!(presets[2].value, "true".to_string());
        assert_eq!(
            deployment.select_tokens,
            Some(vec![GuiSelectTokensCfg {
                key: "test-token".to_string(),
                name: Some("Test name".to_string()),
                description: Some("Test description".to_string()),
            }])
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
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
        let error = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
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
                        wrong: map
"#;
        let error = GuiCfg::parse_from_yaml_optional(
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
                - test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "select-token".to_string(),
                    expected: "a map".to_string()
                },
                location: "select-token index '0' in gui deployment 'deployment1'".to_string(),
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
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("key".to_string()),
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
        let res = GuiCfg::parse_from_yaml_optional(
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
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("deployment1".to_string(), "gui deployment".to_string())
        );
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

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
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

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
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

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
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

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
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

        let keys = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap();
        assert_eq!(keys, vec!["test".to_string(), "test2".to_string()]);
    }

    #[test]
    fn test_check_gui_key_exists() {
        let yaml = r#"
        gui:
            name: test
            description: test
        "#;
        let res = GuiCfg::check_gui_key_exists(vec![get_document(yaml)]).unwrap();
        assert!(res);

        let yaml = r#"
        test: test
        "#;
        let res = GuiCfg::check_gui_key_exists(vec![get_document(yaml)]).unwrap();
        assert!(!res);
    }
}
