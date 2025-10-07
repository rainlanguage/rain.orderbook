use crate::yaml::{require_string, FieldErrorKind, YamlError, YamlParsableString};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;

#[derive(Clone, Debug)]
pub struct SpecVersion;

const CURRENT_SPEC_VERSION: &str = "3";

impl SpecVersion {
    pub fn current() -> String {
        CURRENT_SPEC_VERSION.to_string()
    }

    pub fn is_current(version: &str) -> bool {
        version == CURRENT_SPEC_VERSION
    }
}

impl YamlParsableString for SpecVersion {
    fn parse_from_yaml(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Result<String, YamlError> {
        if documents.is_empty() {
            return Err(YamlError::EmptyFile);
        }

        let mut parsed_version: Option<String> = None;

        for (index, document) in documents.iter().enumerate() {
            let location = if index == 0 {
                "root".to_string()
            } else {
                format!("document {}", index + 1)
            };

            let version = {
                let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
                require_string(&document_read, Some("version"), Some(location.clone()))?
            };

            if let Some(existing_version) = &parsed_version {
                if existing_version != &version {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "version".to_string(),
                            reason: format!(
                                "spec version mismatch: expected '{}', found '{}'",
                                existing_version, version
                            ),
                        },
                        location,
                    });
                }
            } else {
                parsed_version = Some(version);
            }
        }

        parsed_version.ok_or_else(|| YamlError::Field {
            kind: FieldErrorKind::Missing("version".to_string()),
            location: "root".to_string(),
        })
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
        assert!(SpecVersion::is_current("3"));
        assert!(!SpecVersion::is_current("2"));
    }

    #[test]
    fn test_current() {
        assert_eq!(SpecVersion::current(), "3");
    }

    #[test]
    fn test_parse_from_yaml_missing_spec_version() {
        let yaml = r#"
test: test
"#;

        let error = SpecVersion::parse_from_yaml(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("version".to_string()),
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

    #[test]
    fn test_parse_from_yaml_consistent_versions() {
        let documents = vec![
            get_document(
                r#"
version: "3"
"#,
            ),
            get_document(
                r#"
version: "3"
"#,
            ),
        ];

        let version = SpecVersion::parse_from_yaml(documents).unwrap();
        assert_eq!(version, "3");
    }

    #[test]
    fn test_parse_from_yaml_missing_version() {
        let documents = vec![
            get_document(
                r#"
version: "3"
"#,
            ),
            get_document(
                r#"
name: test
"#,
            ),
        ];

        let error = SpecVersion::parse_from_yaml(documents).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("version".to_string()),
                location: "document 2".to_string()
            }
        )
    }

    #[test]
    fn test_parse_from_yaml_mismatched_versions() {
        let documents = vec![
            get_document(
                r#"
version: "3"
"#,
            ),
            get_document(
                r#"
version: "2"
"#,
            ),
        ];

        let error = SpecVersion::parse_from_yaml(documents).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "version".to_string(),
                    reason: "spec version mismatch: expected '3', found '2'".to_string()
                },
                location: "document 2".to_string()
            }
        )
    }
}
