pub mod cache;
pub mod context;
pub mod dotrain;
pub mod orderbook;

use crate::{
    NetworkCfg, ParseDeployerConfigSourceError, ParseDeploymentConfigSourceError,
    ParseNetworkConfigSourceError, ParseOrderConfigSourceError, ParseOrderbookConfigSourceError,
    ParseScenarioConfigSourceError, ParseTokenConfigSourceError,
};
use alloy::primitives::ruint::ParseError as RuintParseError;
use context::{Context, ContextError};
use dotrain::DotrainYaml;
use orderbook::OrderbookYaml;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYamlEmitter;
use strict_yaml_rust::{
    strict_yaml::{Array, Hash},
    EmitError, ScanError, StrictYaml, StrictYamlLoader,
};
use thiserror::Error;
use url::ParseError as UrlParseError;

pub trait YamlParsable: Sized {
    fn new(sources: Vec<String>, validate: bool) -> Result<Self, YamlError>;

    fn from_orderbook_yaml(orderbook_yaml: OrderbookYaml) -> Self;
    fn from_dotrain_yaml(dotrain_yaml: DotrainYaml) -> Self;

    fn get_yaml_string(document: Arc<RwLock<StrictYaml>>) -> Result<String, YamlError> {
        let document = document.read().unwrap();
        let mut out_str = String::new();
        let mut emitter = StrictYamlEmitter::new(&mut out_str);
        emitter.dump(&document)?;

        let out_str = if out_str.starts_with("---") {
            out_str.trim_start_matches("---").trim_start().to_string()
        } else {
            out_str
        };

        Ok(out_str)
    }
}

pub trait YamlParsableHash: Sized + Clone {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError>;

    fn parse_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        key: &str,
        context: Option<&Context>,
    ) -> Result<Self, YamlError> {
        let all = Self::parse_all_from_yaml(documents, context)?;
        all.get(key)
            .ok_or_else(|| YamlError::KeyNotFound(key.to_string()))
            .cloned()
    }
}

pub trait YamlParsableVector: Sized {
    fn parse_all_from_yaml(document: Arc<RwLock<StrictYaml>>) -> Result<Vec<Self>, YamlError>;
}

pub trait YamlParsableString {
    fn parse_from_yaml(document: Arc<RwLock<StrictYaml>>) -> Result<String, YamlError>;

    fn parse_from_yaml_optional(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<Option<String>, YamlError>;
}

pub trait YamlParseableValue: Sized {
    fn parse_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<Self, YamlError>;

    fn parse_from_yaml_optional(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<Option<Self>, YamlError>;
}

pub trait ContextProvider {
    fn create_context(&self) -> Context {
        Context::new()
    }

    fn expand_context_with_remote_networks(&self, context: &mut Context) {
        context.set_remote_networks(self.get_remote_networks_from_cache());
    }
    fn get_remote_networks_from_cache(&self) -> HashMap<String, NetworkCfg>;
}

#[derive(Debug, Error, PartialEq)]
pub enum FieldErrorKind {
    #[error("Missing required field '{0}'")]
    Missing(String),

    #[error("Field '{field}' must be {expected}")]
    InvalidType { field: String, expected: String },

    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },
}

#[derive(Debug, Error)]
pub enum YamlError {
    #[error(transparent)]
    ScanError(#[from] ScanError),
    #[error(transparent)]
    EmitError(#[from] EmitError),
    #[error(transparent)]
    UrlParseError(#[from] UrlParseError),
    #[error(transparent)]
    RuintParseError(#[from] RuintParseError),

    #[error("{kind} in {location}")]
    Field {
        kind: FieldErrorKind,
        location: String,
    },

    #[error("YAML parse error: {0}")]
    ParseError(String),

    #[error("Key '{0}' not found")]
    KeyNotFound(String),
    #[error("Key '{0}' is already defined")]
    KeyShadowing(String),

    #[error("Failed to acquire read lock")]
    ReadLockError,
    #[error("Failed to acquire write lock")]
    WriteLockError,

    #[error("YAML file is empty")]
    EmptyFile,

    #[error("Error while converting to YAML string")]
    ConvertError,
    #[error("Invalid trait function")]
    InvalidTraitFunction,

    #[error(transparent)]
    ParseNetworkConfigSourceError(#[from] ParseNetworkConfigSourceError),
    #[error(transparent)]
    ParseTokenConfigSourceError(#[from] ParseTokenConfigSourceError),
    #[error(transparent)]
    ParseOrderbookConfigSourceError(#[from] ParseOrderbookConfigSourceError),
    #[error(transparent)]
    ParseDeployerConfigSourceError(#[from] ParseDeployerConfigSourceError),
    #[error(transparent)]
    ParseOrderConfigSourceError(#[from] ParseOrderConfigSourceError),
    #[error(transparent)]
    ParseScenarioConfigSourceError(#[from] ParseScenarioConfigSourceError),
    #[error(transparent)]
    ParseDeploymentConfigSourceError(#[from] ParseDeploymentConfigSourceError),
    #[error(transparent)]
    ContextError(#[from] ContextError),
}

impl PartialEq for YamlError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Field {
                    kind: k1,
                    location: l1,
                },
                Self::Field {
                    kind: k2,
                    location: l2,
                },
            ) => k1 == k2 && l1 == l2,
            (Self::ParseError(s1), Self::ParseError(s2)) => s1 == s2,
            (Self::KeyNotFound(k1), Self::KeyNotFound(k2)) => k1 == k2,
            (Self::KeyShadowing(k1), Self::KeyShadowing(k2)) => k1 == k2,
            (Self::ReadLockError, Self::ReadLockError) => true,
            (Self::WriteLockError, Self::WriteLockError) => true,
            (Self::EmptyFile, Self::EmptyFile) => true,
            (Self::ConvertError, Self::ConvertError) => true,
            (Self::InvalidTraitFunction, Self::InvalidTraitFunction) => true,
            // For external error types, we'll compare their string representations
            (Self::ScanError(e1), Self::ScanError(e2)) => e1.to_string() == e2.to_string(),
            (Self::EmitError(e1), Self::EmitError(e2)) => e1.to_string() == e2.to_string(),
            (Self::UrlParseError(e1), Self::UrlParseError(e2)) => e1.to_string() == e2.to_string(),
            (Self::RuintParseError(e1), Self::RuintParseError(e2)) => {
                e1.to_string() == e2.to_string()
            }
            (Self::ParseNetworkConfigSourceError(e1), Self::ParseNetworkConfigSourceError(e2)) => {
                e1 == e2
            }
            (Self::ParseTokenConfigSourceError(e1), Self::ParseTokenConfigSourceError(e2)) => {
                e1 == e2
            }
            (
                Self::ParseOrderbookConfigSourceError(e1),
                Self::ParseOrderbookConfigSourceError(e2),
            ) => e1 == e2,
            (
                Self::ParseDeployerConfigSourceError(e1),
                Self::ParseDeployerConfigSourceError(e2),
            ) => e1 == e2,
            (Self::ParseOrderConfigSourceError(e1), Self::ParseOrderConfigSourceError(e2)) => {
                e1 == e2
            }
            (
                Self::ParseScenarioConfigSourceError(e1),
                Self::ParseScenarioConfigSourceError(e2),
            ) => e1 == e2,
            (
                Self::ParseDeploymentConfigSourceError(e1),
                Self::ParseDeploymentConfigSourceError(e2),
            ) => e1 == e2,
            (Self::ContextError(e1), Self::ContextError(e2)) => e1.to_string() == e2.to_string(),
            _ => false,
        }
    }
}

pub fn load_yaml(yaml: &str) -> Result<StrictYaml, YamlError> {
    let docs = StrictYamlLoader::load_from_str(yaml)?;
    if docs.is_empty() {
        return Err(YamlError::EmptyFile);
    }
    Ok(docs[0].clone())
}

pub fn require_string(
    value: &StrictYaml,
    field: Option<&str>,
    location: Option<String>,
) -> Result<String, YamlError> {
    match field {
        Some(field) => {
            if value[field].is_badvalue() {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing(field.to_string()),
                    location: location.unwrap_or_else(|| "document".to_string()),
                });
            }
            value[field]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: field.to_string(),
                        expected: "a string".to_string(),
                    },
                    location: location.unwrap_or_else(|| "document".to_string()),
                })
        }
        None => value
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "value".to_string(),
                    expected: "a string".to_string(),
                },
                location: location.unwrap_or_else(|| "document".to_string()),
            }),
    }
}
pub fn optional_string(value: &StrictYaml, field: &str) -> Option<String> {
    value[field].as_str().map(|s| s.to_string())
}

pub fn require_hash<'a>(
    value: &'a StrictYaml,
    field: Option<&str>,
    location: Option<String>,
) -> Result<&'a Hash, YamlError> {
    match field {
        Some(field) => {
            if value[field].is_badvalue() {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing(field.to_string()),
                    location: location.unwrap_or_else(|| "document".to_string()),
                });
            }
            value[field].as_hash().ok_or_else(|| YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: field.to_string(),
                    expected: "a map".to_string(),
                },
                location: location.unwrap_or_else(|| "document".to_string()),
            })
        }
        None => value.as_hash().ok_or_else(|| YamlError::Field {
            kind: FieldErrorKind::InvalidType {
                field: "value".to_string(),
                expected: "a map".to_string(),
            },
            location: location.unwrap_or_else(|| "document".to_string()),
        }),
    }
}
pub fn optional_hash<'a>(value: &'a StrictYaml, field: &str) -> Option<&'a Hash> {
    value[field].as_hash()
}

pub fn get_hash_value<'a>(
    hash: &'a Hash,
    field: &str,
    location: Option<String>,
) -> Result<&'a StrictYaml, YamlError> {
    hash.get(&StrictYaml::String(field.to_string()))
        .ok_or_else(|| YamlError::Field {
            kind: FieldErrorKind::Missing(field.to_string()),
            location: location.unwrap_or_else(|| "document".to_string()),
        })
}

pub fn get_hash_value_as_option<'a>(hash: &'a Hash, field: &str) -> Option<&'a StrictYaml> {
    hash.get(&StrictYaml::String(field.to_string()))
}

pub fn require_vec<'a>(
    value: &'a StrictYaml,
    field: &str,
    location: Option<String>,
) -> Result<&'a Array, YamlError> {
    if value[field].is_badvalue() {
        return Err(YamlError::Field {
            kind: FieldErrorKind::Missing(field.to_string()),
            location: location.unwrap_or_else(|| "document".to_string()),
        });
    }
    value[field].as_vec().ok_or_else(|| YamlError::Field {
        kind: FieldErrorKind::InvalidType {
            field: field.to_string(),
            expected: "a vector".to_string(),
        },
        location: location.unwrap_or_else(|| "document".to_string()),
    })
}
pub fn optional_vec<'a>(value: &'a StrictYaml, field: &str) -> Option<&'a Array> {
    value[field].as_vec()
}

pub fn default_document() -> Arc<RwLock<StrictYaml>> {
    Arc::new(RwLock::new(StrictYaml::String("".to_string())))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
    }
}
