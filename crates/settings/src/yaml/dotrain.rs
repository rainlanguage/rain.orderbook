use crate::yaml::get_hash_value_as_option;

use super::{
    get_hash_value, optional_hash, optional_string, optional_vec, require_hash, require_string,
    require_vec, YamlError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strict_yaml_rust::{StrictYaml, StrictYamlLoader};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct IOYaml {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OrderYaml {
    pub inputs: Vec<IOYaml>,
    pub outputs: Vec<IOYaml>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderbook: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct MetricYaml {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_suffix: Option<String>,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ChartYaml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub plots: Option<Vec<PlotYaml>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<MetricYaml>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DeploymentYaml {
    pub scenario: String,
    pub order: String,
}

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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DotrainYaml {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub orders: HashMap<String, OrderYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub scenarios: HashMap<String, ScenarioYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub charts: HashMap<String, ChartYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub deployments: HashMap<String, DeploymentYaml>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gui: Option<GuiYaml>,
}

impl DotrainYaml {
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

    pub fn from_str(yaml: &str) -> Result<Self, YamlError> {
        let docs = StrictYamlLoader::load_from_str(yaml)?;

        if docs.is_empty() {
            return Err(YamlError::EmptyFile);
        }

        let doc = &docs[0];
        let mut yaml = Self::default();

        for (key, value) in
            require_hash(doc, Some("orders"), Some(format!("missing field orders")))?
        {
            let key = key.as_str().unwrap_or_default();
            let order = OrderYaml {
                inputs: require_vec(
                    value,
                    "inputs",
                    Some(format!("inputs missing for order {:?}", key)),
                )?
                .iter()
                .enumerate()
                .map(|(i, input)| {
                    Ok::<IOYaml, YamlError>(IOYaml {
                        token: require_string(
                            input,
                            Some("token"),
                            Some(format!(
                                "token missing in input index {:?} for order {:?}",
                                i, key
                            )),
                        )?,
                        vault_id: optional_string(input, "vault-id"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
                outputs: require_vec(
                    value,
                    "outputs",
                    Some(format!("outputs missing for order {:?}", key)),
                )?
                .iter()
                .enumerate()
                .map(|(i, output)| {
                    Ok::<IOYaml, YamlError>(IOYaml {
                        token: require_string(
                            output,
                            Some("token"),
                            Some(format!(
                                "token missing in output index {:?} for order {:?}",
                                i, key
                            )),
                        )?,
                        vault_id: optional_string(output, "vault-id"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
                deployer: optional_string(value, "deployer"),
                orderbook: optional_string(value, "orderbook"),
            };
            yaml.orders.insert(key.to_string(), order);
        }

        for (key, value) in require_hash(
            doc,
            Some("scenarios"),
            Some(format!("missing field scenarios")),
        )? {
            let key = key.as_str().unwrap_or_default();
            let scenario = Self::parse_scenario(key, value)?;
            yaml.scenarios.insert(key.to_string(), scenario);
        }

        for (key, value) in
            require_hash(doc, Some("charts"), Some(format!("missing field charts")))?
        {
            let key = key.as_str().unwrap_or_default();
            let mut chart = ChartYaml {
                scenario: optional_string(value, "scenario"),
                // plots: None,
                metrics: None,
            };

            // if let Some(plots) = optional_hash(value, "plots") {
            //     let mut plots_map = HashMap::new();
            //     for (plot_key, plot_value) in plots {
            //         let plot_key = plot_key.as_str().unwrap_or_default();
            //         plots_map.insert(
            //             plot_key.to_string(),
            //             require_string(
            //                 plot_value,
            //                 None,
            //                 Some(format!(
            //                     "plot value must be a string for key: {:?}",
            //                     plot_key
            //                 )),
            //             )?,
            //         );
            //     }
            //     chart.plots = Some(plots_map);
            // }

            if let Some(metrics) = optional_vec(value, "metrics") {
                chart.metrics = Some(
                    metrics
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let metric_value = require_hash(
                                v,
                                None,
                                Some(format!(
                                    "metric value must be a map for index {:?} in chart {:?}",
                                    i, key
                                )),
                            )?;
                            Ok(MetricYaml {
                                label: require_string(
                                    get_hash_value(
                                        metric_value,
                                        "label",
                                        Some(format!(
                                            "label missing for metric index {:?} in chart {:?}",
                                            i, key
                                        )),
                                    )?,
                                    None,
                                    Some(format!(
                                        "label must be string for metric index {:?} in chart {:?}",
                                        i, key
                                    )),
                                )?,
                                description: get_hash_value_as_option(metric_value, "description")
                                    .map(|v| {
                                        require_string(
                                            v,
                                            None,
                                            Some(format!(
                                                "description must be string for metric index {:?} in chart {:?}",
                                                i, key
                                            )),
                                        )
                                    })
                                    .transpose()?,
                                    unit_prefix: get_hash_value_as_option(metric_value, "unit_prefix")
                                    .map(|v| {
                                        require_string(
                                            v,
                                            None,
                                            Some(format!(
                                                "unit_prefix must be string for metric index {:?} in chart {:?}",
                                                i, key
                                            )),
                                        )
                                    })
                                    .transpose()?,
                                    unit_suffix: get_hash_value_as_option(metric_value, "unit_suffix")
                                    .map(|v| {
                                        require_string(
                                            v,
                                            None,
                                            Some(format!(
                                                "unit_suffix must be string for metric index {:?} in chart {:?}",
                                                i, key
                                            )),
                                        )
                                    })
                                    .transpose()?,
                                    value: require_string(
                                        get_hash_value(
                                            metric_value,
                                            "value",
                                            Some(format!(
                                                "value missing for metric index {:?} in chart {:?}",
                                                i, key
                                            )),
                                        )?,
                                        None,
                                        Some(format!(
                                            "value must be string for metric index {:?} in chart {:?}",
                                            i, key
                                        )),
                                    )?,
                                precision: get_hash_value_as_option(metric_value, "precision")
                                .map(|v| {
                                    require_string(
                                        v,
                                        None,
                                        Some(format!(
                                            "precision must be string for metric index {:?} in chart {:?}",
                                            i, key
                                        )),
                                    )
                                })
                                .transpose()?,
                            })
                        })
                        .collect::<Result<Vec<_>, YamlError>>()?,
                );
            }
            yaml.charts.insert(key.to_string(), chart);
        }

        for (key, value) in require_hash(
            doc,
            Some("deployments"),
            Some(format!("missing field deployments")),
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
            yaml.deployments.insert(key.to_string(), deployment);
        }

        if let Some(gui) = optional_hash(doc, "gui") {
            let name = gui
                .get(&StrictYaml::String("name".to_string()))
                .ok_or(YamlError::ParseError(format!("name missing for gui")))?
                .as_str()
                .ok_or(YamlError::ParseError(format!("name must be a string")))?
                .to_string();
            let description = gui
                .get(&StrictYaml::String("description".to_string()))
                .ok_or(YamlError::ParseError(format!(
                    "description missing for gui"
                )))?
                .as_str()
                .ok_or(YamlError::ParseError(format!(
                    "description must be a string"
                )))?
                .to_string();
            let deployments = gui
                .get(&StrictYaml::String("deployments".to_string()))
                .ok_or(YamlError::ParseError(format!(
                    "deployments missing for gui"
                )))?
                .as_vec()
                .ok_or(YamlError::ParseError(format!(
                    "deployments must be a vector"
                )))?;

            let gui_deployments = deployments
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    Ok(GuiDeploymentYaml {
                        deployment: require_string(
                            value,
                            Some("deployment"),
                            Some(format!(
                                "deployment missing for gui deployment index {:?}",
                                i
                            )),
                        )?,
                        name: require_string(
                            value,
                            Some("name"),
                            Some(format!(
                                "name missing for gui deployment index {:?}",
                                i
                            )),
                        )?,
                        description: require_string(
                            value,
                            Some("description"),
                            Some(format!(
                                "description missing for gui deployment index {:?}",
                                i
                            )),
                        )?,
                        deposits: require_vec(
                            value,
                            "deposits",
                            Some(format!(
                                "deposits missing for gui deployment index {:?}",
                                i
                            )),
                        )?
                        .iter()
                        .enumerate()
                        .map(|(deposit_i, deposit_value)| {
                            Ok(GuiDepositYaml {
                                token: require_string(
                                    deposit_value,
                                    Some("token"),
                                    Some(format!(
                                        "token missing for deposit index {:?} in gui deployment index {:?}",
                                        deposit_i, i
                                    )),
                                )?,
                                presets: require_vec(
                                    deposit_value,
                                    "presets",
                                    Some(format!(
                                        "presets missing for deposit index {:?} in gui deployment index {:?}",
                                        deposit_i, i
                                    )),
                                )?
                                .iter()
                                .enumerate()
                                .map(|(preset_i, p)| {
                                    Ok(p.as_str().ok_or(YamlError::ParseError(format!(
                                        "preset value must be a string for preset index {:?} in deposit index {:?} in gui deployment index {:?}",
                                        preset_i, deposit_i, i
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
                                "fields missing for gui deployment index {:?}",
                                i
                            )),
                        )?
                        .iter()
                        .enumerate()
                        .map(|(field_i, field_value)| {
                            Ok(GuiFieldDefinitionYaml {
                                binding: require_string(
                                    field_value,
                                    Some("binding"),
                                    Some(format!(
                                        "binding missing for field index {:?} in gui deployment index {:?}",
                                        field_i, i
                                    )),
                                )?,
                                name: require_string(
                                    field_value,
                                    Some("name"),
                                    Some(format!(
                                        "name missing for field index {:?} in gui deployment index {:?}",
                                        field_i, i
                                    )),
                                )?,
                                description: optional_string(field_value, "description"),
                                presets: match optional_vec(field_value, "presets") {
                                    Some(p) => Some(p.iter().enumerate().map(|(preset_i, preset_value)| {
                                        println!("preset_value: {:?}", preset_value);
                                        Ok(GuiPresetYaml {
                                            name: optional_string(preset_value, "name"),
                                            value: require_string(preset_value, Some("value"), Some(format!(
                                                "preset value must be a string for preset index {:?} in field index {:?} in gui deployment index {:?}",
                                                preset_i, field_i, i
                                            )))?,
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
                                    .map(|(select_token_i, select_token_value)| {
                                        Ok(select_token_value.as_str().ok_or(YamlError::ParseError(format!(
                                            "select-token value must be a string for select-token index {:?} in gui deployment index {:?}",
                                            select_token_i, i
                                        )))?.to_string())
                                    })
                                    .collect::<Result<Vec<_>, YamlError>>()?,
                            ),
                            None => None,
                        },
                    })
                })
                .collect::<Result<Vec<_>, YamlError>>()?;

            yaml.gui = Some(GuiYaml {
                name,
                description,
                deployments: gui_deployments,
            });
        }

        Ok(yaml)
    }

    pub fn set_order(&mut self, key: String, order: OrderYaml) {
        self.orders.insert(key, order);
    }

    pub fn set_scenario(&mut self, key: String, scenario: ScenarioYaml) {
        self.scenarios.insert(key, scenario);
    }

    pub fn set_chart(&mut self, key: String, chart: ChartYaml) {
        self.charts.insert(key, chart);
    }

    pub fn set_deployment(&mut self, key: String, deployment: DeploymentYaml) {
        self.deployments.insert(key, deployment);
    }

    pub fn set_gui(&mut self, gui: GuiYaml) {
        self.gui = Some(gui);
    }
}
