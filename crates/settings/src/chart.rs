use crate::{
    yaml::{
        context::Context, default_document, get_hash_value, get_hash_value_as_option,
        optional_hash, optional_string, optional_vec, require_hash, require_string, require_vec,
        FieldErrorKind, YamlError, YamlParsableHash,
    },
    *,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ChartCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub scenario: Arc<ScenarioCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub plots: Option<Vec<PlotCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub metrics: Option<Vec<MetricCfg>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ChartCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct MetricCfg {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_suffix: Option<String>,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<u8>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(MetricCfg);

impl ChartCfg {
    pub fn validate_u32(value: String, field: String, location: String) -> Result<u32, YamlError> {
        value.parse::<u32>().map_err(|e| YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field,
                reason: e.to_string(),
            },
            location: location.clone(),
        })
    }
}

impl YamlParsableHash for ChartCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut charts = HashMap::new();

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(charts_hash) = optional_hash(&document_read, "charts") {
                for (key_yaml, chart_yaml) in charts_hash {
                    let chart_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("chart '{chart_key}'");

                    let scenario_key =
                        optional_string(chart_yaml, "scenario").unwrap_or(chart_key.clone());
                    let scenario =
                        ScenarioCfg::parse_from_yaml(documents.clone(), &scenario_key, context)?;

                    let mut chart = ChartCfg {
                        document: document.clone(),
                        key: chart_key.clone(),
                        scenario: Arc::new(scenario),
                        plots: None,
                        metrics: None,
                    };

                    chart.plots = if let Some(plots) = optional_hash(chart_yaml, "plots") {
                        let mut plots_vec = Vec::new();
                        for (plot_key_yaml, plot_yaml) in plots.iter() {
                            let plot_key = plot_key_yaml.as_str().unwrap_or_default().to_string();
                            let sub_location = format!("plot key '{plot_key}' in {location}");

                            let title = optional_string(plot_yaml, "title");
                            let subtitle = optional_string(plot_yaml, "subtitle");

                            let marks =
                                require_vec(plot_yaml, "marks", Some(sub_location.clone()))?
                                    .iter()
                                    .enumerate()
                                    .map(|(mark_index, mark_yaml)| {
                                        let mark_location =
                                            format!("mark index '{mark_index}' in {sub_location}");
                                        let mark_map = require_hash(
                                            mark_yaml,
                                            None,
                                            Some(mark_location.clone()),
                                        )?;

                                        let mark_options_map = get_hash_value(
                                            mark_map,
                                            "options",
                                            Some(mark_location.clone()),
                                        )?
                                        .as_hash()
                                        .ok_or(YamlError::Field {
                                            kind: FieldErrorKind::Missing("options".to_string()),
                                            location: mark_location.clone(),
                                        })?;

                                        let transform = if let Some(transform_yaml) =
                                            get_hash_value_as_option(mark_options_map, "transform")
                                        {
                                            let transform_location =
                                                format!("transform in {mark_location}");

                                            let transform_hash = require_hash(
                                                transform_yaml,
                                                None,
                                                Some(transform_location.clone()),
                                            )?;

                                            let transform_type = get_hash_value(
                                                transform_hash,
                                                "type",
                                                Some(transform_location.clone()),
                                            )?
                                            .as_str()
                                            .ok_or(YamlError::Field {
                                                kind: FieldErrorKind::Missing("type".to_string()),
                                                location: transform_location.clone(),
                                            })?;

                                            let content_hash = get_hash_value(
                                                transform_hash,
                                                "content",
                                                Some(transform_location.clone()),
                                            )?
                                            .as_hash()
                                            .ok_or(YamlError::Field {
                                                kind: FieldErrorKind::Missing(
                                                    "content".to_string(),
                                                ),
                                                location: transform_location.clone(),
                                            })?;
                                            let content_location =
                                                format!("content in {transform_location}");

                                            let outputs_hash = get_hash_value(
                                                content_hash,
                                                "outputs",
                                                Some(content_location.clone()),
                                            )?
                                            .as_hash()
                                            .ok_or(YamlError::Field {
                                                kind: FieldErrorKind::Missing(
                                                    "outputs".to_string(),
                                                ),
                                                location: content_location.clone(),
                                            })?;
                                            let outputs_location =
                                                format!("outputs in {content_location}");

                                            let outputs_x =
                                                get_hash_value_as_option(outputs_hash, "x")
                                                    .map(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .map(|s| s.to_string());
                                            let outputs_y =
                                                get_hash_value_as_option(outputs_hash, "y")
                                                    .map(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .map(|s| s.to_string());
                                            let outputs_r =
                                                get_hash_value_as_option(outputs_hash, "r")
                                                    .map(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .map(|s| {
                                                        ChartCfg::validate_u32(
                                                            s.to_string(),
                                                            "r".to_string(),
                                                            outputs_location.clone(),
                                                        )
                                                    })
                                                    .transpose()?;
                                            let outputs_z =
                                                get_hash_value_as_option(outputs_hash, "z")
                                                    .map(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .map(|s| s.to_string());
                                            let outputs_stroke =
                                                get_hash_value_as_option(outputs_hash, "stroke")
                                                    .map(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .map(|s| s.to_string());
                                            let outputs_fill =
                                                get_hash_value_as_option(outputs_hash, "fill")
                                                    .map(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .map(|s| s.to_string());

                                            let options_hash = get_hash_value(
                                                content_hash,
                                                "options",
                                                Some(content_location.clone()),
                                            )?
                                            .as_hash()
                                            .ok_or(YamlError::Field {
                                                kind: FieldErrorKind::Missing(
                                                    "options".to_string(),
                                                ),
                                                location: content_location.clone(),
                                            })?;
                                            let options_location =
                                                format!("options in {content_location}");

                                            let transform = match transform_type {
                                                "hexbin" => {
                                                    let options_x =
                                                        get_hash_value_as_option(options_hash, "x")
                                                            .map(|s| s.as_str())
                                                            .unwrap_or_default()
                                                            .map(|s| s.to_string());
                                                    let options_y =
                                                        get_hash_value_as_option(options_hash, "y")
                                                            .map(|s| s.as_str())
                                                            .unwrap_or_default()
                                                            .map(|s| s.to_string());
                                                    let options_bin_width =
                                                        get_hash_value_as_option(
                                                            options_hash,
                                                            "bin-width",
                                                        )
                                                        .map(|s| s.as_str())
                                                        .unwrap_or_default()
                                                        .map(|s| {
                                                            ChartCfg::validate_u32(
                                                                s.to_string(),
                                                                "bin-width".to_string(),
                                                                options_location.clone(),
                                                            )
                                                        })
                                                        .transpose()?;

                                                    TransformCfg::HexBin(HexBinTransformCfg {
                                                        outputs: TransformOutputsCfg {
                                                            x: outputs_x,
                                                            y: outputs_y,
                                                            r: outputs_r,
                                                            z: outputs_z,
                                                            stroke: outputs_stroke,
                                                            fill: outputs_fill,
                                                        },
                                                        options: HexBinOptionsCfg {
                                                            x: options_x,
                                                            y: options_y,
                                                            bin_width: options_bin_width,
                                                        },
                                                    })
                                                }
                                                "binx" => {
                                                    let options_x =
                                                        get_hash_value_as_option(options_hash, "x")
                                                            .map(|s| s.as_str())
                                                            .unwrap_or_default()
                                                            .map(|s| s.to_string());
                                                    let options_thresholds =
                                                        get_hash_value_as_option(
                                                            options_hash,
                                                            "thresholds",
                                                        )
                                                        .map(|s| s.as_str())
                                                        .unwrap_or_default()
                                                        .map(|s| {
                                                            ChartCfg::validate_u32(
                                                                s.to_string(),
                                                                "thresholds".to_string(),
                                                                options_location.clone(),
                                                            )
                                                        })
                                                        .transpose()?;

                                                    TransformCfg::BinX(BinXTransformCfg {
                                                        outputs: TransformOutputsCfg {
                                                            x: outputs_x,
                                                            y: outputs_y,
                                                            r: outputs_r,
                                                            z: outputs_z,
                                                            stroke: outputs_stroke,
                                                            fill: outputs_fill,
                                                        },
                                                        options: BinXOptionsCfg {
                                                            x: options_x,
                                                            thresholds: options_thresholds,
                                                        },
                                                    })
                                                }
                                                _ => {
                                                    return Err(YamlError::Field {
                                                        kind: FieldErrorKind::InvalidValue {
                                                            field: "type".to_string(),
                                                            reason: format!(
                                                    "invalid transform type: '{transform_type}'"
                                                ),
                                                        },
                                                        location: transform_location.clone(),
                                                    })
                                                }
                                            };

                                            Some(transform)
                                        } else {
                                            None
                                        };

                                        let mark_type = get_hash_value(
                                            mark_map,
                                            "type",
                                            Some(mark_location.clone()),
                                        )?
                                        .as_str()
                                        .ok_or(YamlError::Field {
                                            kind: FieldErrorKind::Missing("type".to_string()),
                                            location: mark_location.clone(),
                                        })?;

                                        let mark = match mark_type {
                                            "dot" | "line" => {
                                                let x =
                                                    get_hash_value_as_option(mark_options_map, "x")
                                                        .map(|s| s.as_str())
                                                        .unwrap_or_default()
                                                        .map(|s| s.to_string());
                                                let y =
                                                    get_hash_value_as_option(mark_options_map, "y")
                                                        .map(|s| s.as_str())
                                                        .unwrap_or_default()
                                                        .map(|s| s.to_string());
                                                let r =
                                                    get_hash_value_as_option(mark_options_map, "r")
                                                        .map(|s| s.as_str())
                                                        .unwrap_or_default()
                                                        .map(|s| {
                                                            ChartCfg::validate_u32(
                                                                s.to_string(),
                                                                "r".to_string(),
                                                                mark_location.clone(),
                                                            )
                                                        })
                                                        .transpose()?;
                                                let fill = get_hash_value_as_option(
                                                    mark_options_map,
                                                    "fill",
                                                )
                                                .map(|s| s.as_str())
                                                .unwrap_or_default()
                                                .map(|s| s.to_string());
                                                let stroke = get_hash_value_as_option(
                                                    mark_options_map,
                                                    "stroke",
                                                )
                                                .map(|s| s.as_str())
                                                .unwrap_or_default()
                                                .map(|s| s.to_string());

                                                if mark_type == "dot" {
                                                    MarkCfg::Dot(DotOptionsCfg {
                                                        x,
                                                        y,
                                                        r,
                                                        fill,
                                                        stroke,
                                                        transform,
                                                    })
                                                } else {
                                                    MarkCfg::Line(LineOptionsCfg {
                                                        x,
                                                        y,
                                                        r,
                                                        fill,
                                                        stroke,
                                                        transform,
                                                    })
                                                }
                                            }
                                            "recty" => {
                                                let x0 = get_hash_value_as_option(
                                                    mark_options_map,
                                                    "x0",
                                                )
                                                .map(|s| s.as_str())
                                                .unwrap_or_default()
                                                .map(|s| s.to_string());
                                                let x1 = get_hash_value_as_option(
                                                    mark_options_map,
                                                    "x1",
                                                )
                                                .map(|s| s.as_str())
                                                .unwrap_or_default()
                                                .map(|s| s.to_string());
                                                let y0 = get_hash_value_as_option(
                                                    mark_options_map,
                                                    "y0",
                                                )
                                                .map(|s| s.as_str())
                                                .unwrap_or_default()
                                                .map(|s| s.to_string());
                                                let y1 = get_hash_value_as_option(
                                                    mark_options_map,
                                                    "y1",
                                                )
                                                .map(|s| s.as_str())
                                                .unwrap_or_default()
                                                .map(|s| s.to_string());

                                                MarkCfg::RectY(RectYOptionsCfg {
                                                    x0,
                                                    x1,
                                                    y0,
                                                    y1,
                                                    transform,
                                                })
                                            }
                                            _ => {
                                                return Err(YamlError::Field {
                                                    kind: FieldErrorKind::InvalidValue {
                                                        field: "type".to_string(),
                                                        reason: format!(
                                                            "invalid mark type: '{mark_type}'"
                                                        ),
                                                    },
                                                    location: mark_location.clone(),
                                                })
                                            }
                                        };

                                        Ok(mark)
                                    })
                                    .collect::<Result<Vec<_>, YamlError>>()?;

                            let x = if let Some(x_options_hash) = optional_hash(plot_yaml, "x") {
                                let label = get_hash_value_as_option(x_options_hash, "label")
                                    .map(|s| s.as_str())
                                    .unwrap_or_default();
                                let anchor = get_hash_value_as_option(x_options_hash, "anchor")
                                    .map(|s| s.as_str())
                                    .unwrap_or_default();
                                let label_anchor =
                                    get_hash_value_as_option(x_options_hash, "label-anchor")
                                        .map(|s| s.as_str())
                                        .unwrap_or_default();
                                let label_arrow =
                                    get_hash_value_as_option(x_options_hash, "label-arrow")
                                        .map(|s| s.as_str())
                                        .unwrap_or_default();
                                Some(AxisOptionsCfg {
                                    label: label.map(|s| s.to_string()),
                                    anchor: anchor.map(|s| s.to_string()),
                                    label_anchor: label_anchor.map(|s| s.to_string()),
                                    label_arrow: label_arrow.map(|s| s.to_string()),
                                })
                            } else {
                                None
                            };

                            let y = if let Some(y_options_hash) = optional_hash(plot_yaml, "y") {
                                let label = get_hash_value_as_option(y_options_hash, "label")
                                    .map(|s| s.as_str())
                                    .unwrap_or_default();
                                let anchor = get_hash_value_as_option(y_options_hash, "anchor")
                                    .map(|s| s.as_str())
                                    .unwrap_or_default();
                                let label_anchor =
                                    get_hash_value_as_option(y_options_hash, "label-anchor")
                                        .map(|s| s.as_str())
                                        .unwrap_or_default();
                                let label_arrow =
                                    get_hash_value_as_option(y_options_hash, "label-arrow")
                                        .map(|s| s.as_str())
                                        .unwrap_or_default();
                                Some(AxisOptionsCfg {
                                    label: label.map(|s| s.to_string()),
                                    anchor: anchor.map(|s| s.to_string()),
                                    label_anchor: label_anchor.map(|s| s.to_string()),
                                    label_arrow: label_arrow.map(|s| s.to_string()),
                                })
                            } else {
                                None
                            };

                            let margin = optional_string(plot_yaml, "margin")
                                .map(|s| {
                                    ChartCfg::validate_u32(
                                        s,
                                        "margin".to_string(),
                                        sub_location.clone(),
                                    )
                                })
                                .transpose()?;
                            let margin_left = optional_string(plot_yaml, "margin-left")
                                .map(|s| {
                                    ChartCfg::validate_u32(
                                        s,
                                        "margin-left".to_string(),
                                        sub_location.clone(),
                                    )
                                })
                                .transpose()?;
                            let margin_right = optional_string(plot_yaml, "margin-right")
                                .map(|s| {
                                    ChartCfg::validate_u32(
                                        s,
                                        "margin-right".to_string(),
                                        sub_location.clone(),
                                    )
                                })
                                .transpose()?;
                            let margin_top = optional_string(plot_yaml, "margin-top")
                                .map(|s| {
                                    ChartCfg::validate_u32(
                                        s,
                                        "margin-top".to_string(),
                                        sub_location.clone(),
                                    )
                                })
                                .transpose()?;
                            let margin_bottom = optional_string(plot_yaml, "margin-bottom")
                                .map(|s| {
                                    ChartCfg::validate_u32(
                                        s,
                                        "margin-bottom".to_string(),
                                        sub_location.clone(),
                                    )
                                })
                                .transpose()?;
                            let inset = optional_string(plot_yaml, "inset")
                                .map(|s| {
                                    ChartCfg::validate_u32(
                                        s,
                                        "inset".to_string(),
                                        sub_location.clone(),
                                    )
                                })
                                .transpose()?;

                            plots_vec.push(PlotCfg {
                                title,
                                subtitle,
                                marks,
                                x,
                                y,
                                margin,
                                margin_left,
                                margin_right,
                                margin_top,
                                margin_bottom,
                                inset,
                            });
                        }
                        Some(plots_vec)
                    } else {
                        None
                    };

                    chart.metrics = if let Some(metrics) = optional_vec(chart_yaml, "metrics") {
                        let mut metrics_vec = Vec::new();
                        for (metric_index, metric) in metrics.iter().enumerate() {
                            let sub_location =
                                format!("metric index '{metric_index}' in {location}");

                            let label =
                                require_string(metric, Some("label"), Some(sub_location.clone()))?;
                            let description = optional_string(metric, "description");
                            let unit_prefix = optional_string(metric, "unit-prefix");
                            let unit_suffix = optional_string(metric, "unit-suffix");
                            let value =
                                require_string(metric, Some("value"), Some(sub_location.clone()))?;
                            let precision = optional_string(metric, "precision")
                                .map(|s| {
                                    s.parse::<u8>().map_err(|e| YamlError::Field {
                                        kind: FieldErrorKind::InvalidValue {
                                            field: "precision".to_string(),
                                            reason: e.to_string(),
                                        },
                                        location: sub_location.clone(),
                                    })
                                })
                                .transpose()?;

                            metrics_vec.push(MetricCfg {
                                label,
                                description,
                                unit_prefix,
                                unit_suffix,
                                value,
                                precision,
                            });
                        }
                        Some(metrics_vec)
                    } else {
                        None
                    };

                    if charts.contains_key(&chart_key) {
                        return Err(YamlError::KeyShadowing(
                            chart_key.clone(),
                            "charts".to_string(),
                        ));
                    }
                    charts.insert(chart_key.clone(), chart);
                }
            }
        }

        Ok(charts)
    }
}

impl Default for ChartCfg {
    fn default() -> Self {
        ChartCfg {
            document: default_document(),
            key: "".to_string(),
            scenario: Arc::new(ScenarioCfg::default()),
            plots: None,
            metrics: None,
        }
    }
}
impl PartialEq for ChartCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.scenario == other.scenario
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseChartConfigSourceError {
    #[error("Scenario not found: {0}")]
    ScenarioNotFoundError(String),
}

impl ChartConfigSource {
    pub fn try_into_chart(
        self,
        name: String,
        scenarios: &HashMap<String, Arc<ScenarioCfg>>,
    ) -> Result<ChartCfg, ParseChartConfigSourceError> {
        let scenario_ref = match self.scenario {
            Some(scenario_name) => scenarios
                .get(&scenario_name)
                .ok_or(ParseChartConfigSourceError::ScenarioNotFoundError(
                    scenario_name.clone(),
                ))
                .map(Arc::clone)?,
            None => scenarios
                .get(&name)
                .ok_or(ParseChartConfigSourceError::ScenarioNotFoundError(
                    name.clone(),
                ))
                .map(Arc::clone)?,
        };

        // Convert `self.plots` from Option<HashMap<String, Plot>> to Option<Vec<Plot>>
        let plots = self.plots.map(|plots_map| {
            plots_map
                .into_iter()
                .map(|(name, mut plot)| {
                    // If the plot does not have a title, use the name from the map
                    plot.title.get_or_insert(name);
                    plot
                })
                .collect::<Vec<PlotCfg>>()
        });

        Ok(ChartCfg {
            document: default_document(),
            key: name,
            scenario: scenario_ref,
            metrics: self.metrics,
            plots,
        })
    }
}

#[cfg(test)]
mod tests {
    use strict_yaml_rust::StrictYaml;

    use crate::test::mock_plot;
    use crate::yaml::tests::get_document;

    use self::test::mock_deployer;

    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    fn create_scenario(name: &str, runs: Option<u64>) -> (String, Arc<ScenarioCfg>) {
        let scenario = ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: name.into(),
            bindings: HashMap::from([(String::from("key"), String::from("value"))]), // Example binding
            runs,
            blocks: None,
            deployer: mock_deployer(),
        };
        (name.to_string(), Arc::new(scenario))
    }

    #[test]
    fn test_success_explicit_scenario_name() {
        let (scenario_name, scenario) = create_scenario("scenario1", 100.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(scenario_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartConfigSource {
            scenario: Some(scenario_name),
            plots: Some(plots),
            metrics: None,
        };

        let chart = chart_string
            .try_into_chart("chart1".to_string(), &scenarios)
            .unwrap();
        assert!(Arc::ptr_eq(
            &chart.scenario,
            scenarios.get("scenario1").unwrap()
        ));
    }

    #[test]
    fn test_success_using_chart_name() {
        let (chart_name, scenario) = create_scenario("chart2", 100.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(chart_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartConfigSource {
            scenario: None,
            plots: Some(plots),
            metrics: None,
        };

        let chart = chart_string
            .try_into_chart(chart_name.clone(), &scenarios)
            .unwrap();
        assert!(Arc::ptr_eq(
            &chart.scenario,
            scenarios.get(&chart_name).unwrap()
        ));
    }

    #[test]
    fn test_scenario_not_found_error() {
        let scenarios = HashMap::<String, Arc<ScenarioCfg>>::new(); // No scenarios added

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let chart_string = ChartConfigSource {
            scenario: Some("nonexistent_scenario".to_string()),
            plots: Some(plots),
            metrics: None,
        };

        let result = chart_string.try_into_chart("chart3".to_string(), &scenarios);
        assert!(matches!(
            result,
            Err(ParseChartConfigSourceError::ScenarioNotFoundError(_))
        ));
    }

    #[test]
    fn test_no_scenario_matching_chart_name() {
        let scenarios = HashMap::<String, Arc<ScenarioCfg>>::new(); // No scenarios added

        let chart_string = ChartConfigSource {
            scenario: None,
            plots: None,
            metrics: None,
        };

        let result = chart_string.try_into_chart("chart4".to_string(), &scenarios);
        assert!(matches!(
            result,
            Err(ParseChartConfigSourceError::ScenarioNotFoundError(_))
        ));
    }

    #[test]
    fn test_multiple_plots() {
        let (scenario_name, scenario) = create_scenario("scenario5", 200.into());
        let mut scenarios = HashMap::new();
        scenarios.insert(scenario_name.clone(), scenario);

        let mut plots = HashMap::new();
        let (plot_name, plot) = mock_plot("plot1");
        plots.insert(plot_name, plot);

        let (plot_name, plot) = mock_plot("plot2");
        plots.insert(plot_name, plot);

        let metrics: Vec<MetricCfg> = vec![MetricCfg {
            label: "label".to_string(),
            description: Some("description".to_string()),
            unit_prefix: Some("unit_prefix".to_string()),
            unit_suffix: Some("unit_suffix".to_string()),
            value: "value".to_string(),
            precision: Some(2),
        }];

        let chart_string = ChartConfigSource {
            scenario: Some(scenario_name),
            plots: Some(plots),
            metrics: Some(metrics),
        };

        let chart = chart_string
            .try_into_chart("chart5".to_string(), &scenarios)
            .unwrap();
        assert!(Arc::ptr_eq(
            &chart.scenario,
            scenarios.get("scenario5").unwrap()
        ));
        assert_eq!(chart.clone().plots.unwrap().len(), 2);

        // both plots should have the name "Title"
        let mut plots = chart
            .plots
            .unwrap()
            .iter()
            .map(|p| p.title.clone())
            .collect::<Vec<Option<String>>>();
        plots.sort();
        assert_eq!(
            plots,
            vec![Some("Title".to_string()), Some("Title".to_string())]
        );
    }

    const PREFIX: &str = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
deployers:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
scenarios:
    chart1:
        deployer: mainnet
        bindings:
            key1: binding1
"#;

    #[test]
    fn test_parse_charts_from_yaml_transform() {
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        transform:
                            test: test
                      
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("type".to_string()),
                location: "transform in mark index '0' in plot key 'plot1' in chart 'chart1'"
                    .to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        transform:
                            type: test
                            content:
                                outputs:
                                    x: 1
                                options:
                                    x: 1
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "type".to_string(),
                    reason: "invalid transform type: 'test'".to_string(),
                },
                location: "transform in mark index '0' in plot key 'plot1' in chart 'chart1'"
                    .to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        transform:
                            type: hexbin
                      
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("content".to_string()),
                location: "transform in mark index '0' in plot key 'plot1' in chart 'chart1'"
                    .to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        transform:
                            type: hexbin
                            content:
                                type: type
                      
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("outputs".to_string()),
                location:
                    "content in transform in mark index '0' in plot key 'plot1' in chart 'chart1'"
                        .to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        transform:
                            type: hexbin
                            content:
                                outputs:
                                    r: a
                      
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "r".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location:
                    "outputs in content in transform in mark index '0' in plot key 'plot1' in chart 'chart1'"
                        .to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        transform:
                            type: hexbin
                            content:
                                outputs:
                                    r: 1
                                options:
                                    bin-width: a
                      
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "bin-width".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location:
                    "options in content in transform in mark index '0' in plot key 'plot1' in chart 'chart1'"
                        .to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        transform:
                            type: binx
                            content:
                                outputs:
                                    r: 1
                                options:
                                    thresholds: a
                      
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "thresholds".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location:
                    "options in content in transform in mark index '0' in plot key 'plot1' in chart 'chart1'"
                        .to_string(),
            }
        );
    }

    #[test]
    fn test_parse_charts_from_yaml_marks() {
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                test: test
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("marks".to_string()),
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks: test
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "marks".to_string(),
                    expected: "a vector".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    test: test
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "marks".to_string(),
                    expected: "a vector".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - test: test
                      options:
                        x: 1
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("type".to_string()),
                location: "mark index '0' in plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: test
                      options:
                        x: 1
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "type".to_string(),
                    reason: "invalid mark type: 'test'".to_string(),
                },
                location: "mark index '0' in plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      test: test
                      
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("options".to_string()),
                location: "mark index '0' in plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_charts_from_yaml_margin() {
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        x: 1
                        y: 1
                margin: a
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "margin".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        x: 1
                        y: 1
                margin: 1
                margin-left: a
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "margin-left".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        x: 1
                        y: 1
                margin: 1
                margin-left: 1
                margin-right: a
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "margin-right".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        x: 1
                        y: 1
                margin: 1
                margin-left: 1
                margin-right: 1
                margin-top: a
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "margin-top".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        x: 1
                        y: 1
                margin: 1
                margin-left: 1
                margin-right: 1
                margin-top: 1
                margin-bottom: a
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "margin-bottom".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                marks:
                    - type: line
                      options:
                        x: 1
                        y: 1
                margin: 1
                margin-left: 1
                margin-right: 1
                margin-top: 1
                margin-bottom: 1
                inset: a
"#;
        let error = ChartCfg::parse_all_from_yaml(
            vec![get_document(format!("{}\n{}", PREFIX, yaml).as_str())],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "inset".to_string(),
                    reason: "invalid digit found in string".to_string(),
                },
                location: "plot key 'plot1' in chart 'chart1'".to_string(),
            }
        );
    }
}
