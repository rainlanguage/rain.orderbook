use crate::yaml::{require_string, YamlError, YamlParsableString};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;

#[derive(Clone, Debug)]
pub struct SpecVersion;

const CURRENT_SPEC_VERSION: &str = "1";

impl SpecVersion {
    pub fn current() -> String {
        CURRENT_SPEC_VERSION.to_string()
    }

    pub fn is_current(version: &str) -> bool {
        version == CURRENT_SPEC_VERSION
    }
}

impl YamlParsableString for SpecVersion {
    fn parse_from_yaml(document: Arc<RwLock<StrictYaml>>) -> Result<String, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let value = require_string(
            &document_read,
            Some("spec-version"),
            Some("root".to_string()),
        )?;
        Ok(value)
    }

    fn parse_from_yaml_optional(_: Arc<RwLock<StrictYaml>>) -> Result<Option<String>, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{tests::get_document, FieldErrorKind};

    #[test]
    fn test_is_current() {
        assert!(SpecVersion::is_current("1"));
        assert!(!SpecVersion::is_current("2"));
    }

    #[test]
    fn test_current() {
        assert_eq!(SpecVersion::current(), "1");
    }

    #[test]
    fn test_parse_from_yaml_missing_spec_version() {
        let yaml = r#"
test: test
"#;

        let error = SpecVersion::parse_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("spec-version".to_string()),
                location: "root".to_string()
            }
        )
    }

    #[test]
    fn test_parse_from_yaml_optional() {
        let yaml = r#"
test: test
"#;
        let error = SpecVersion::parse_from_yaml_optional(get_document(yaml)).unwrap_err();
        assert_eq!(error, YamlError::InvalidTraitFunction);
    }
}
