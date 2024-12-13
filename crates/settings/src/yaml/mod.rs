pub mod orderbook;

use crate::{ParseNetworkConfigSourceError, ParseTokenConfigSourceError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};
use strict_yaml_rust::{
    strict_yaml::{Array, Hash},
    EmitError, ScanError, StrictYaml, StrictYamlLoader,
};
use thiserror::Error;
use url::ParseError as UrlParseError;

pub trait YamlParsableHash: Sized + Clone {
    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Self>, YamlError>;

    fn parse_from_yaml(document: Arc<RwLock<StrictYaml>>, key: String) -> Result<Self, YamlError> {
        let all = Self::parse_all_from_yaml(document)?;
        all.get(&key)
            .ok_or_else(|| YamlError::KeyNotFound(key))
            .cloned()
    }
}

pub trait YamlParsableVector: Sized {
    fn parse_all_from_yaml(document: Arc<RwLock<StrictYaml>>) -> Result<Vec<Self>, YamlError>;
}

pub trait YamlParsableString {
    fn parse_from_yaml(document: Arc<RwLock<StrictYaml>>) -> Result<String, YamlError>;
}

#[derive(Debug, Error)]
pub enum YamlError {
    #[error(transparent)]
    ScanError(#[from] ScanError),
    #[error(transparent)]
    EmitError(#[from] EmitError),
    #[error(transparent)]
    RwLockReadGuardError(#[from] PoisonError<RwLockReadGuard<'static, StrictYaml>>),
    #[error(transparent)]
    RwLockWriteGuardError(#[from] PoisonError<RwLockWriteGuard<'static, StrictYaml>>),
    #[error(transparent)]
    UrlParseError(#[from] UrlParseError),
    #[error("Yaml file is empty")]
    EmptyFile,
    #[error("Yaml parse error: {0}")]
    ParseError(String),
    #[error("Missing custom message")]
    MissingCustomMsg,
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Error while converting to yaml string")]
    ConvertError,
    #[error("Document read lock error")]
    ReadLockError,
    #[error("Document write lock error")]
    WriteLockError,
    #[error(transparent)]
    ParseNetworkConfigSourceError(#[from] ParseNetworkConfigSourceError),
    #[error(transparent)]
    ParseTokenConfigSourceError(#[from] ParseTokenConfigSourceError),
}
impl PartialEq for YamlError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ScanError(a), Self::ScanError(b)) => a == b,
            (Self::EmptyFile, Self::EmptyFile) => true,
            (Self::ParseError(a), Self::ParseError(b)) => a == b,
            (Self::MissingCustomMsg, Self::MissingCustomMsg) => true,
            (Self::KeyNotFound(a), Self::KeyNotFound(b)) => a == b,
            (Self::ConvertError, Self::ConvertError) => true,
            (Self::ReadLockError, Self::ReadLockError) => true,
            (Self::WriteLockError, Self::WriteLockError) => true,
            (Self::ParseNetworkConfigSourceError(a), Self::ParseNetworkConfigSourceError(b)) => {
                a == b
            }
            (Self::ParseTokenConfigSourceError(a), Self::ParseTokenConfigSourceError(b)) => a == b,
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
    custom_msg: Option<String>,
) -> Result<String, YamlError> {
    match field {
        Some(field) => value[field].as_str().map(|s| s.to_string()).ok_or_else(|| {
            YamlError::ParseError(custom_msg.unwrap_or(format!("{field} must be a string")))
        }),
        None => value
            .as_str()
            .map(|s| s.to_string())
            .ok_or(YamlError::ParseError(
                custom_msg.ok_or(YamlError::MissingCustomMsg)?,
            )),
    }
}
pub fn optional_string(value: &StrictYaml, field: &str) -> Option<String> {
    value[field].as_str().map(|s| s.to_string())
}

pub fn require_hash<'a>(
    value: &'a StrictYaml,
    field: Option<&str>,
    custom_msg: Option<String>,
) -> Result<&'a Hash, YamlError> {
    match field {
        Some(field) => value[field].as_hash().ok_or_else(|| {
            YamlError::ParseError(custom_msg.unwrap_or(format!("{field} must be a map")))
        }),
        None => value.as_hash().ok_or(YamlError::ParseError(
            custom_msg.ok_or(YamlError::MissingCustomMsg)?,
        )),
    }
}
pub fn optional_hash<'a>(value: &'a StrictYaml, field: &str) -> Option<&'a Hash> {
    value[field].as_hash()
}

pub fn get_hash_value<'a>(
    hash: &'a Hash,
    field: &str,
    custom_msg: Option<String>,
) -> Result<&'a StrictYaml, YamlError> {
    hash.get(&StrictYaml::String(field.to_string()))
        .ok_or(YamlError::ParseError(
            custom_msg.unwrap_or(format!("{field} missing in map")),
        ))
}

pub fn get_hash_value_as_option<'a>(hash: &'a Hash, field: &str) -> Option<&'a StrictYaml> {
    hash.get(&StrictYaml::String(field.to_string()))
}

pub fn require_vec<'a>(
    value: &'a StrictYaml,
    field: &str,
    custom_msg: Option<String>,
) -> Result<&'a Array, YamlError> {
    value[field].as_vec().ok_or_else(|| {
        YamlError::ParseError(custom_msg.unwrap_or(format!("{field} must be a vector")))
    })
}
pub fn optional_vec<'a>(value: &'a StrictYaml, field: &str) -> Option<&'a Array> {
    value[field].as_vec()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
    }
}
