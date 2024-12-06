use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DeploymentYaml {
    pub scenario: String,
    pub order: String,
}
impl DeploymentYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, Self>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut deployments = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("deployments"),
            Some("missing field deployments".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let deployment = DeploymentYaml {
                scenario: require_string(
                    value,
                    Some("scenario"),
                    Some(format!("scenario missing for deployment {:?}", key)),
                )?,
                order: require_string(
                    value,
                    Some("order"),
                    Some(format!("order missing for deployment {:?}", key)),
                )?,
            };
            deployments.insert(key.to_string(), deployment);
        }
        Ok(deployments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployments_errors() {
        let yaml = r#"
test: test
"#;
        let error = DeploymentYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field deployments".to_string())
        );

        let yaml = r#"
deployments:
    deployment1:
        test: test
        "#;
        let error = DeploymentYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("scenario missing for deployment \"deployment1\"".to_string())
        );

        let yaml = r#"
deployments:
    deployment1:
        scenario: scenario1
        test: test
        "#;
        let error = DeploymentYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("order missing for deployment \"deployment1\"".to_string())
        );
    }
}
