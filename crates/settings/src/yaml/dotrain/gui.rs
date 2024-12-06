use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GuiPresetYaml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GuiDepositYaml {
    pub token: String,
    pub presets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GuiFieldDefinitionYaml {
    pub binding: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presets: Option<Vec<GuiPresetYaml>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GuiDeploymentYaml {
    pub deployment: String,
    pub name: String,
    pub description: String,
    pub deposits: Vec<GuiDepositYaml>,
    pub fields: Vec<GuiFieldDefinitionYaml>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_tokens: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GuiYaml {
    pub name: String,
    pub description: String,
    pub deployments: Vec<GuiDeploymentYaml>,
}
impl GuiYaml {
    pub fn try_from_string(source: &str) -> Result<Option<Self>, YamlError> {
        let doc = &load_yaml(source)?;

        if let Some(gui) = optional_hash(doc, "gui") {
            let name = require_string(
                get_hash_value(gui, "name", Some("name field missing in gui".to_string()))?,
                None,
                Some("name field must be a string in gui".to_string()),
            )?;
            let description = require_string(
                get_hash_value(
                    gui,
                    "description",
                    Some("description field missing in gui".to_string()),
                )?,
                None,
                Some("description field must be a string in gui".to_string()),
            )?;
            let deployments = gui
                .get(&StrictYaml::String("deployments".to_string()))
                .ok_or(YamlError::ParseError(
                    "deployments field missing in gui".to_string(),
                ))?
                .as_vec()
                .ok_or(YamlError::ParseError(
                    "deployments field must be a list in gui".to_string(),
                ))?;

            let gui_deployments = deployments
                .iter()
                .enumerate()
                .map(|(deployment_index, value)| {
                    Ok(GuiDeploymentYaml {
                        deployment: require_string(
                            value,
                            Some("deployment"),
                            Some(format!(
                                "deployment string missing for gui deployment index: {}",
                                deployment_index
                            )),
                        )?,
                        name: require_string(
                            value,
                            Some("name"),
                            Some(format!(
                                "name string missing for gui deployment index: {}",
                                deployment_index
                            )),
                        )?,
                        description: require_string(
                            value,
                            Some("description"),
                            Some(format!(
                                "description string missing for gui deployment index: {}",
                                deployment_index
                            )),
                        )?,
                        deposits: require_vec(
                            value,
                            "deposits",
                            Some(format!(
                                "deposits list missing for gui deployment index: {}",
                                deployment_index
                            )),
                        )?
                        .iter()
                        .enumerate()
                        .map(|(deposit_index, deposit_value)| {
                            Ok(GuiDepositYaml {
                                token: require_string(
                                    deposit_value,
                                    Some("token"),
                                    Some(format!(
                                        "token string missing for deposit index: {} for gui deployment index: {}",
                                        deposit_index, deployment_index
                                    )),
                                )?,
                                presets: require_vec(
                                    deposit_value,
                                    "presets",
                                    Some(format!(
                                        "presets list missing for deposit index: {} for gui deployment index: {}",
                                        deposit_index, deployment_index
                                    )),
                                )?
                                .iter()
                                .enumerate()
                                .map(|(preset_i, p)| {
                                    Ok(p.as_str().ok_or(YamlError::ParseError(format!(
                                        "preset value must be a string for preset list index: {} for deposit index: {} for gui deployment index: {}",
                                        preset_i, deposit_index, deployment_index
                                    )))?.to_string())
                                })
                                .collect::<Result<Vec<_>, YamlError>>()?,
                            })
                        })
                        .collect::<Result<Vec<_>, YamlError>>()?,
                        fields: require_vec(
                            value,
                            "fields",
                            Some(format!(
                                "fields list missing for gui deployment index: {}",
                                deployment_index
                            )),
                        )?
                        .iter()
                        .enumerate()
                        .map(|(field_index, field_value)| {
                            Ok(GuiFieldDefinitionYaml {
                                binding: require_string(
                                    field_value,
                                    Some("binding"),
                                    Some(format!(
                                        "binding string missing for field index: {} for gui deployment index: {}",
                                        field_index, deployment_index
                                    )),
                                )?,
                                name: require_string(
                                    field_value,
                                    Some("name"),
                                    Some(format!(
                                        "name string missing for field index: {} for gui deployment index: {}",
                                        field_index, deployment_index
                                    )),
                                )?,
                                description: optional_string(field_value, "description"),
                                presets: match optional_vec(field_value, "presets") {
                                    Some(p) => Some(p.iter().enumerate().map(|(preset_index, preset_value)| {
                                        Ok(GuiPresetYaml {
                                            name: optional_string(preset_value, "name"),
                                            value: require_string(
                                                preset_value,
                                                Some("value"),
                                                Some(format!(
                                                    "preset value must be a string for preset index: {} for field index: {} for gui deployment index: {}",
                                                    preset_index, field_index, deployment_index
                                                ))
                                            )?,
                                        })
                                    })
                                    .collect::<Result<Vec<_>, YamlError>>()?),
                                    None => None,
                                },
                            })
                        })
                        .collect::<Result<Vec<_>, YamlError>>()?,
                        select_tokens: match optional_vec(value, "select-tokens") {
                            Some(tokens) => Some(
                                tokens
                                    .iter()
                                    .enumerate()
                                    .map(|(select_token_index, select_token_value)| {
                                        Ok(select_token_value.as_str().ok_or(YamlError::ParseError(format!(
                                            "select-token value must be a string for select-token index: {} for gui deployment index: {}",
                                            select_token_index, deployment_index
                                        )))?.to_string())
                                    })
                                    .collect::<Result<Vec<_>, YamlError>>()?,
                            ),
                            None => None,
                        },
                    })
                })
                .collect::<Result<Vec<_>, YamlError>>()?;

            Ok(Some(GuiYaml {
                name,
                description,
                deployments: gui_deployments,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_errors() {
        let yaml = r#"
gui:
    test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name field missing in gui".to_string())
        );
        let yaml = r#"
gui:
    name:
      - test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name field must be a string in gui".to_string())
        );
        let yaml = r#"
gui:
    name:
      - test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name field must be a string in gui".to_string())
        );

        let yaml = r#"
gui:
    name: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("description field missing in gui".to_string())
        );
        let yaml = r#"
gui:
    name: test
    description:
      - test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("description field must be a string in gui".to_string())
        );
        let yaml = r#"
gui:
    name: test
    description:
      - test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("description field must be a string in gui".to_string())
        );

        let yaml = r#"
gui:
    name: test
    description: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deployments field missing in gui".to_string())
        );
        let yaml = r#"
gui:
    name: test
    description: test
    deployments: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deployments field must be a list in gui".to_string())
        );
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deployments field must be a list in gui".to_string())
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "deployment string missing for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("name string missing for gui deployment index: 0".to_string())
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "description string missing for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("deposits list missing for gui deployment index: 0".to_string())
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token string missing for deposit index: 0 for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "presets list missing for deposit index: 0 for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "preset value must be a string for preset list index: 0 for deposit index: 0 for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("fields list missing for gui deployment index: 0".to_string())
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "binding string missing for field index: 0 for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - binding: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "name string missing for field index: 0 for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - binding: test
              name: test
              presets:
                - value:
                    - test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "preset value must be a string for preset index: 0 for field index: 0 for gui deployment index: 0".to_string()
            )
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        - deployment: deployment1
          name: test
          description: test
          deposits:
            - token: eth
              presets:
                - 1
          fields:
            - binding: test
              name: test
              presets:
                - value: test
          select-tokens:
            - test: test
"#;
        let error = GuiYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "select-token value must be a string for select-token index: 0 for gui deployment index: 0".to_string()
            )
        );
    }
}
