use crate::ParseNetworkYamlError;
use strict_yaml_rust::{
    strict_yaml::{Array, Hash},
    ScanError, StrictYaml, StrictYamlLoader,
};
use thiserror::Error;

pub mod dotrain;
pub mod orderbook;

#[derive(Debug, Error, PartialEq)]
pub enum YamlError {
    #[error(transparent)]
    ScanError(#[from] ScanError),
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
    #[error(transparent)]
    ParseNetworkYamlError(#[from] ParseNetworkYamlError),
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
