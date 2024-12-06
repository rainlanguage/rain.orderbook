use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ScenarioYaml {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bindings: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenarios: Option<HashMap<String, ScenarioYaml>>,
}
impl ScenarioYaml {
    fn parse_scenario(key: &str, value: &StrictYaml) -> Result<ScenarioYaml, YamlError> {
        Ok(ScenarioYaml {
            bindings: require_hash(
                value,
                Some("bindings"),
                Some(format!("bindings missing for scenario {:?}", key)),
            )?
            .iter()
            .map(|(binding_key, binding_value)| {
                let binding_key = binding_key.as_str().unwrap_or_default();
                Ok((
                    binding_key.to_string(),
                    require_string(
                        binding_value,
                        None,
                        Some(format!(
                            "binding value must be a string for key {:?}",
                            binding_key
                        )),
                    )?,
                ))
            })
            .collect::<Result<HashMap<_, _>, YamlError>>()?,
            runs: optional_string(value, "runs"),
            blocks: optional_string(value, "blocks"),
            deployer: optional_string(value, "deployer"),
            scenarios: match optional_hash(value, "scenarios") {
                Some(scenarios) => {
                    let mut scenarios_map = HashMap::new();
                    for (sub_key, sub_value) in scenarios {
                        let sub_key = sub_key.as_str().unwrap_or_default();
                        let sub_scenario = Self::parse_scenario(key, sub_value)?;
                        scenarios_map.insert(sub_key.to_string(), sub_scenario);
                    }
                    Some(scenarios_map)
                }
                None => None,
            },
        })
    }

    pub fn try_from_string(source: &str) -> Result<HashMap<String, Self>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut scenarios = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("scenarios"),
            Some("missing field scenarios".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let scenario = Self::parse_scenario(key, value)?;
            scenarios.insert(key.to_string(), scenario);
        }
        Ok(scenarios)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenarios_errors() {
        let yaml = r#"
test: test
"#;
        let error = ScenarioYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field scenarios".to_string())
        );

        let yaml = r#"
scenarios:
    scenario1:
        test: test
"#;
        let error = ScenarioYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("bindings missing for scenario \"scenario1\"".to_string())
        );

        let yaml = r#"
scenarios:
    scenario1:
        bindings:
            key1:
              - value1
"#;
        let error = ScenarioYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("binding value must be a string for key \"key1\"".to_string())
        );

        let yaml = r#"
scenarios:
    scenario1:
        bindings:
            key1:
              - value1: value2
"#;
        let error = ScenarioYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("binding value must be a string for key \"key1\"".to_string())
        );
    }
}
