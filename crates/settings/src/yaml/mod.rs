use strict_yaml_rust::{
    strict_yaml::{Array, Hash},
    ScanError, StrictYaml,
};
use thiserror::Error;

pub mod dotrain;
pub mod orderbook;

#[cfg(test)]
mod dotrain_test;
#[cfg(test)]
mod orderbook_test;

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
    field: &str,
    custom_msg: Option<String>,
) -> Result<&'a Hash, YamlError> {
    value[field].as_hash().ok_or_else(|| {
        YamlError::ParseError(custom_msg.unwrap_or(format!("{field} must be a map")))
    })
}
pub fn optional_hash<'a>(value: &'a StrictYaml, field: &str) -> Option<&'a Hash> {
    value[field].as_hash()
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
