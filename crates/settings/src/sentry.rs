use crate::yaml::{optional_string, YamlError, YamlParsableString};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;

#[derive(Clone, Debug)]
pub struct YamlSentry;

impl YamlParsableString for YamlSentry {
    fn parse_from_yaml(_: Arc<RwLock<StrictYaml>>) -> Result<String, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }

    fn parse_from_yaml_optional(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<Option<String>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

        if let Some(value) = optional_string(&document_read, "sentry") {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
