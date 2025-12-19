use crate::{
    yaml::{
        context::Context, default_document, get_hash_value, get_hash_value_as_option,
        optional_hash, optional_string, optional_vec, require_hash, require_string, require_vec,
        FieldErrorKind, YamlError, YamlParsableHash,
    },
    *,
};

const ALLOWED_CHART_KEYS: [&str; 3] = ["metrics", "plots", "scenario"];

const ALLOWED_PLOT_KEYS: [&str; 11] = [
    "inset",
    "margin",
    "margin-bottom",
    "margin-left",
    "margin-right",
    "margin-top",
    "marks",
    "subtitle",
    "title",
    "x",
    "y",
];

const ALLOWED_MARK_KEYS: [&str; 2] = ["options", "type"];

const ALLOWED_MARK_OPTIONS_KEYS: [&str; 10] = [
    "fill",
    "r",
    "stroke",
    "transform",
    "x",
    "x0",
    "x1",
    "y",
    "y0",
    "y1",
];

const ALLOWED_TRANSFORM_KEYS: [&str; 2] = ["content", "type"];

const ALLOWED_TRANSFORM_CONTENT_KEYS: [&str; 2] = ["options", "outputs"];

const ALLOWED_TRANSFORM_OUTPUTS_KEYS: [&str; 6] = ["fill", "r", "stroke", "x", "y", "z"];

const ALLOWED_TRANSFORM_OPTIONS_KEYS: [&str; 4] = ["bin-width", "thresholds", "x", "y"];

const ALLOWED_AXIS_KEYS: [&str; 4] = ["anchor", "label", "label-anchor", "label-arrow"];

const ALLOWED_METRIC_KEYS: [&str; 6] = [
    "description",
    "label",
    "precision",
    "unit-prefix",
    "unit-suffix",
    "value",
];
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
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

    fn sanitize_hash_with_keys(hash: &Hash, allowed_keys: &[&str]) -> Hash {
        let mut sanitized = Hash::new();
        for allowed_key in allowed_keys.iter() {
            let key_yaml = StrictYaml::String(allowed_key.to_string());
            if let Some(v) = hash.get(&key_yaml) {
                sanitized.insert(key_yaml, v.clone());
            }
        }
        sanitized
    }

    fn sanitize_transform_hash(transform_hash: &Hash) -> Hash {
        let mut sanitized = Self::sanitize_hash_with_keys(transform_hash, &ALLOWED_TRANSFORM_KEYS);

        if let Some(StrictYaml::Hash(content_hash)) =
            sanitized.get(&StrictYaml::String("content".to_string()))
        {
            let mut sanitized_content =
                Self::sanitize_hash_with_keys(content_hash, &ALLOWED_TRANSFORM_CONTENT_KEYS);

            if let Some(StrictYaml::Hash(outputs_hash)) =
                sanitized_content.get(&StrictYaml::String("outputs".to_string()))
            {
                let sanitized_outputs =
                    Self::sanitize_hash_with_keys(outputs_hash, &ALLOWED_TRANSFORM_OUTPUTS_KEYS);
                sanitized_content.insert(
                    StrictYaml::String("outputs".to_string()),
                    StrictYaml::Hash(sanitized_outputs),
                );
            }

            if let Some(StrictYaml::Hash(options_hash)) =
                sanitized_content.get(&StrictYaml::String("options".to_string()))
            {
                let sanitized_options =
                    Self::sanitize_hash_with_keys(options_hash, &ALLOWED_TRANSFORM_OPTIONS_KEYS);
                sanitized_content.insert(
                    StrictYaml::String("options".to_string()),
                    StrictYaml::Hash(sanitized_options),
                );
            }

            sanitized.insert(
                StrictYaml::String("content".to_string()),
                StrictYaml::Hash(sanitized_content),
            );
        }

        sanitized
    }

    fn sanitize_mark_options_hash(options_hash: &Hash) -> Hash {
        let mut sanitized = Self::sanitize_hash_with_keys(options_hash, &ALLOWED_MARK_OPTIONS_KEYS);

        if let Some(StrictYaml::Hash(transform_hash)) =
            sanitized.get(&StrictYaml::String("transform".to_string()))
        {
            let sanitized_transform = Self::sanitize_transform_hash(transform_hash);
            sanitized.insert(
                StrictYaml::String("transform".to_string()),
                StrictYaml::Hash(sanitized_transform),
            );
        }

        sanitized
    }

    fn sanitize_mark_hash(mark_hash: &Hash) -> Hash {
        let mut sanitized = Self::sanitize_hash_with_keys(mark_hash, &ALLOWED_MARK_KEYS);

        if let Some(StrictYaml::Hash(options_hash)) =
            sanitized.get(&StrictYaml::String("options".to_string()))
        {
            let sanitized_options = Self::sanitize_mark_options_hash(options_hash);
            sanitized.insert(
                StrictYaml::String("options".to_string()),
                StrictYaml::Hash(sanitized_options),
            );
        }

        sanitized
    }

    fn sanitize_marks_array(marks_array: &[StrictYaml]) -> Vec<StrictYaml> {
        marks_array
            .iter()
            .filter_map(|mark| {
                if let StrictYaml::Hash(mark_hash) = mark {
                    Some(StrictYaml::Hash(Self::sanitize_mark_hash(mark_hash)))
                } else {
                    None
                }
            })
            .collect()
    }

    fn sanitize_axis_hash(axis_hash: &Hash) -> Hash {
        Self::sanitize_hash_with_keys(axis_hash, &ALLOWED_AXIS_KEYS)
    }

    fn sanitize_plot_hash(plot_hash: &Hash) -> Hash {
        let mut sanitized = Self::sanitize_hash_with_keys(plot_hash, &ALLOWED_PLOT_KEYS);

        if let Some(StrictYaml::Array(marks_array)) =
            sanitized.get(&StrictYaml::String("marks".to_string()))
        {
            let sanitized_marks = Self::sanitize_marks_array(marks_array);
            sanitized.insert(
                StrictYaml::String("marks".to_string()),
                StrictYaml::Array(sanitized_marks),
            );
        }

        if let Some(StrictYaml::Hash(x_hash)) = sanitized.get(&StrictYaml::String("x".to_string()))
        {
            let sanitized_x = Self::sanitize_axis_hash(x_hash);
            sanitized.insert(
                StrictYaml::String("x".to_string()),
                StrictYaml::Hash(sanitized_x),
            );
        }

        if let Some(StrictYaml::Hash(y_hash)) = sanitized.get(&StrictYaml::String("y".to_string()))
        {
            let sanitized_y = Self::sanitize_axis_hash(y_hash);
            sanitized.insert(
                StrictYaml::String("y".to_string()),
                StrictYaml::Hash(sanitized_y),
            );
        }

        sanitized
    }

    fn sanitize_plots_hash(plots_hash: &Hash) -> Hash {
        let mut sanitized_plots: Vec<(String, StrictYaml)> = Vec::new();

        for (plot_key, plot_value) in plots_hash {
            let Some(plot_key_str) = plot_key.as_str() else {
                continue;
            };
            let StrictYaml::Hash(ref plot_hash) = *plot_value else {
                continue;
            };

            let sanitized_plot = Self::sanitize_plot_hash(plot_hash);
            sanitized_plots.push((plot_key_str.to_string(), StrictYaml::Hash(sanitized_plot)));
        }
        sanitized_plots.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut new_plots_hash = Hash::new();
        for (key, value) in sanitized_plots {
            new_plots_hash.insert(StrictYaml::String(key), value);
        }
        new_plots_hash
    }

    fn sanitize_metric_hash(metric_hash: &Hash) -> Hash {
        Self::sanitize_hash_with_keys(metric_hash, &ALLOWED_METRIC_KEYS)
    }

    fn sanitize_metrics_array(metrics_array: &[StrictYaml]) -> Vec<StrictYaml> {
        metrics_array
            .iter()
            .filter_map(|metric| {
                if let StrictYaml::Hash(metric_hash) = metric {
                    Some(StrictYaml::Hash(Self::sanitize_metric_hash(metric_hash)))
                } else {
                    None
                }
            })
            .collect()
    }

    fn sanitize_chart_hash(chart_hash: &Hash) -> Hash {
        let mut sanitized = Self::sanitize_hash_with_keys(chart_hash, &ALLOWED_CHART_KEYS);

        if let Some(StrictYaml::Hash(plots_hash)) =
            sanitized.get(&StrictYaml::String("plots".to_string()))
        {
            let sanitized_plots = Self::sanitize_plots_hash(plots_hash);
            sanitized.insert(
                StrictYaml::String("plots".to_string()),
                StrictYaml::Hash(sanitized_plots),
            );
        }

        if let Some(StrictYaml::Array(metrics_array)) =
            sanitized.get(&StrictYaml::String("metrics".to_string()))
        {
            let sanitized_metrics = Self::sanitize_metrics_array(metrics_array);
            sanitized.insert(
                StrictYaml::String("metrics".to_string()),
                StrictYaml::Array(sanitized_metrics),
            );
        }

        sanitized
    }

    fn sanitize_charts_hash(charts_hash: &Hash) -> Hash {
        let mut sanitized_charts: Vec<(String, StrictYaml)> = Vec::new();

        for (chart_key, chart_value) in charts_hash {
            let Some(chart_key_str) = chart_key.as_str() else {
                continue;
            };
            let StrictYaml::Hash(ref chart_hash) = *chart_value else {
                continue;
            };

            let sanitized_chart = Self::sanitize_chart_hash(chart_hash);
            sanitized_charts.push((chart_key_str.to_string(), StrictYaml::Hash(sanitized_chart)));
        }
        sanitized_charts.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut new_charts_hash = Hash::new();
        for (key, value) in sanitized_charts {
            new_charts_hash.insert(StrictYaml::String(key), value);
        }
        new_charts_hash
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

    fn to_yaml_value(&self) -> Result<StrictYaml, YamlError> {
        fn insert_string(hash: &mut Hash, key: &str, value: String) {
            hash.insert(
                StrictYaml::String(key.to_string()),
                StrictYaml::String(value),
            );
        }

        fn insert_option_string(hash: &mut Hash, key: &str, value: &Option<String>) {
            if let Some(value) = value {
                insert_string(hash, key, value.clone());
            }
        }

        fn insert_option_u32(hash: &mut Hash, key: &str, value: Option<u32>) {
            if let Some(value) = value {
                insert_string(hash, key, value.to_string());
            }
        }

        fn insert_option_u8(hash: &mut Hash, key: &str, value: Option<u8>) {
            if let Some(value) = value {
                insert_string(hash, key, value.to_string());
            }
        }

        fn outputs_to_yaml(outputs: &TransformOutputsCfg) -> StrictYaml {
            let mut outputs_hash = Hash::new();
            insert_option_string(&mut outputs_hash, "x", &outputs.x);
            insert_option_string(&mut outputs_hash, "y", &outputs.y);
            insert_option_u32(&mut outputs_hash, "r", outputs.r);
            insert_option_string(&mut outputs_hash, "z", &outputs.z);
            insert_option_string(&mut outputs_hash, "stroke", &outputs.stroke);
            insert_option_string(&mut outputs_hash, "fill", &outputs.fill);

            StrictYaml::Hash(outputs_hash)
        }

        fn hexbin_options_to_yaml(options: &HexBinOptionsCfg) -> StrictYaml {
            let mut options_hash = Hash::new();
            insert_option_string(&mut options_hash, "x", &options.x);
            insert_option_string(&mut options_hash, "y", &options.y);
            insert_option_u32(&mut options_hash, "bin-width", options.bin_width);

            StrictYaml::Hash(options_hash)
        }

        fn binx_options_to_yaml(options: &BinXOptionsCfg) -> StrictYaml {
            let mut options_hash = Hash::new();
            insert_option_string(&mut options_hash, "x", &options.x);
            insert_option_u32(&mut options_hash, "thresholds", options.thresholds);

            StrictYaml::Hash(options_hash)
        }

        fn transform_to_yaml(transform: &TransformCfg) -> StrictYaml {
            let mut transform_hash = Hash::new();

            match transform {
                TransformCfg::HexBin(hexbin) => {
                    insert_string(&mut transform_hash, "type", "hexbin".to_string());

                    let mut content_hash = Hash::new();
                    content_hash.insert(
                        StrictYaml::String("outputs".to_string()),
                        outputs_to_yaml(&hexbin.outputs),
                    );
                    content_hash.insert(
                        StrictYaml::String("options".to_string()),
                        hexbin_options_to_yaml(&hexbin.options),
                    );

                    transform_hash.insert(
                        StrictYaml::String("content".to_string()),
                        StrictYaml::Hash(content_hash),
                    );
                }
                TransformCfg::BinX(binx) => {
                    insert_string(&mut transform_hash, "type", "binx".to_string());

                    let mut content_hash = Hash::new();
                    content_hash.insert(
                        StrictYaml::String("outputs".to_string()),
                        outputs_to_yaml(&binx.outputs),
                    );
                    content_hash.insert(
                        StrictYaml::String("options".to_string()),
                        binx_options_to_yaml(&binx.options),
                    );

                    transform_hash.insert(
                        StrictYaml::String("content".to_string()),
                        StrictYaml::Hash(content_hash),
                    );
                }
            }

            StrictYaml::Hash(transform_hash)
        }

        fn axis_options_to_yaml(axis: &AxisOptionsCfg) -> StrictYaml {
            let mut axis_hash = Hash::new();
            insert_option_string(&mut axis_hash, "label", &axis.label);
            insert_option_string(&mut axis_hash, "anchor", &axis.anchor);
            insert_option_string(&mut axis_hash, "label-anchor", &axis.label_anchor);
            insert_option_string(&mut axis_hash, "label-arrow", &axis.label_arrow);

            StrictYaml::Hash(axis_hash)
        }

        fn dot_line_options_to_yaml(
            x: &Option<String>,
            y: &Option<String>,
            r: &Option<u32>,
            fill: &Option<String>,
            stroke: &Option<String>,
            transform: &Option<TransformCfg>,
        ) -> Hash {
            let mut options_hash = Hash::new();
            insert_option_string(&mut options_hash, "x", x);
            insert_option_string(&mut options_hash, "y", y);
            insert_option_u32(&mut options_hash, "r", *r);
            insert_option_string(&mut options_hash, "fill", fill);
            insert_option_string(&mut options_hash, "stroke", stroke);

            if let Some(transform) = transform {
                options_hash.insert(
                    StrictYaml::String("transform".to_string()),
                    transform_to_yaml(transform),
                );
            }

            options_hash
        }

        fn recty_options_to_yaml(
            x0: &Option<String>,
            x1: &Option<String>,
            y0: &Option<String>,
            y1: &Option<String>,
            transform: &Option<TransformCfg>,
        ) -> Hash {
            let mut options_hash = Hash::new();
            insert_option_string(&mut options_hash, "x0", x0);
            insert_option_string(&mut options_hash, "x1", x1);
            insert_option_string(&mut options_hash, "y0", y0);
            insert_option_string(&mut options_hash, "y1", y1);

            if let Some(transform) = transform {
                options_hash.insert(
                    StrictYaml::String("transform".to_string()),
                    transform_to_yaml(transform),
                );
            }

            options_hash
        }

        let mut chart_yaml = Hash::new();

        insert_string(&mut chart_yaml, "scenario", self.scenario.key.clone());

        if let Some(plots) = &self.plots {
            let mut plots_hash = Hash::new();

            for (index, plot) in plots.iter().enumerate() {
                let mut plot_hash = Hash::new();
                insert_option_string(&mut plot_hash, "title", &plot.title);
                insert_option_string(&mut plot_hash, "subtitle", &plot.subtitle);

                let mut marks_yaml = Vec::new();

                for mark in &plot.marks {
                    let mut mark_hash = Hash::new();

                    match mark {
                        MarkCfg::Dot(options) => {
                            insert_string(&mut mark_hash, "type", "dot".to_string());

                            mark_hash.insert(
                                StrictYaml::String("options".to_string()),
                                StrictYaml::Hash(dot_line_options_to_yaml(
                                    &options.x,
                                    &options.y,
                                    &options.r,
                                    &options.fill,
                                    &options.stroke,
                                    &options.transform,
                                )),
                            );
                        }
                        MarkCfg::Line(options) => {
                            insert_string(&mut mark_hash, "type", "line".to_string());

                            mark_hash.insert(
                                StrictYaml::String("options".to_string()),
                                StrictYaml::Hash(dot_line_options_to_yaml(
                                    &options.x,
                                    &options.y,
                                    &options.r,
                                    &options.fill,
                                    &options.stroke,
                                    &options.transform,
                                )),
                            );
                        }
                        MarkCfg::RectY(options) => {
                            insert_string(&mut mark_hash, "type", "recty".to_string());

                            mark_hash.insert(
                                StrictYaml::String("options".to_string()),
                                StrictYaml::Hash(recty_options_to_yaml(
                                    &options.x0,
                                    &options.x1,
                                    &options.y0,
                                    &options.y1,
                                    &options.transform,
                                )),
                            );
                        }
                    }

                    marks_yaml.push(StrictYaml::Hash(mark_hash));
                }

                plot_hash.insert(
                    StrictYaml::String("marks".to_string()),
                    StrictYaml::Array(marks_yaml),
                );

                if let Some(axis) = &plot.x {
                    plot_hash.insert(
                        StrictYaml::String("x".to_string()),
                        axis_options_to_yaml(axis),
                    );
                }

                if let Some(axis) = &plot.y {
                    plot_hash.insert(
                        StrictYaml::String("y".to_string()),
                        axis_options_to_yaml(axis),
                    );
                }

                insert_option_u32(&mut plot_hash, "margin", plot.margin);
                insert_option_u32(&mut plot_hash, "margin-left", plot.margin_left);
                insert_option_u32(&mut plot_hash, "margin-right", plot.margin_right);
                insert_option_u32(&mut plot_hash, "margin-top", plot.margin_top);
                insert_option_u32(&mut plot_hash, "margin-bottom", plot.margin_bottom);
                insert_option_u32(&mut plot_hash, "inset", plot.inset);

                plots_hash.insert(
                    StrictYaml::String(format!("plot{}", index + 1)),
                    StrictYaml::Hash(plot_hash),
                );
            }

            chart_yaml.insert(
                StrictYaml::String("plots".to_string()),
                StrictYaml::Hash(plots_hash),
            );
        }

        if let Some(metrics) = &self.metrics {
            let metrics_yaml = metrics
                .iter()
                .map(|metric| {
                    let mut metric_hash = Hash::new();

                    insert_string(&mut metric_hash, "label", metric.label.clone());
                    insert_option_string(&mut metric_hash, "description", &metric.description);
                    insert_option_string(&mut metric_hash, "unit-prefix", &metric.unit_prefix);
                    insert_option_string(&mut metric_hash, "unit-suffix", &metric.unit_suffix);
                    insert_string(&mut metric_hash, "value", metric.value.clone());
                    insert_option_u8(&mut metric_hash, "precision", metric.precision);

                    StrictYaml::Hash(metric_hash)
                })
                .collect();

            chart_yaml.insert(
                StrictYaml::String("metrics".to_string()),
                StrictYaml::Array(metrics_yaml),
            );
        }

        Ok(StrictYaml::Hash(chart_yaml))
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let charts_key = StrictYaml::String("charts".to_string());
            let Some(charts_value) = root_hash.get(&charts_key) else {
                continue;
            };
            let StrictYaml::Hash(ref charts_hash) = *charts_value else {
                continue;
            };

            let sanitized = ChartCfg::sanitize_charts_hash(charts_hash);
            root_hash.insert(charts_key, StrictYaml::Hash(sanitized));
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    const PREFIX: &str = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
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

    #[test]
    fn test_to_yaml_hash_serializes_charts() {
        fn sy(value: &str) -> StrictYaml {
            StrictYaml::String(value.to_string())
        }

        let chart_with_optional = ChartCfg {
            document: default_document(),
            key: "chart-with-options".to_string(),
            scenario: Arc::new(ScenarioCfg {
                key: "scenario-a".to_string(),
                ..ScenarioCfg::default()
            }),
            plots: Some(vec![PlotCfg {
                title: Some("Plot Title".to_string()),
                subtitle: Some("Plot Subtitle".to_string()),
                marks: vec![
                    MarkCfg::Dot(DotOptionsCfg {
                        x: Some("x".to_string()),
                        y: Some("y".to_string()),
                        r: Some(7),
                        fill: Some("blue".to_string()),
                        stroke: Some("red".to_string()),
                        transform: Some(TransformCfg::HexBin(HexBinTransformCfg {
                            outputs: TransformOutputsCfg {
                                x: Some("out-x".to_string()),
                                y: None,
                                r: Some(3),
                                z: Some("z-val".to_string()),
                                stroke: None,
                                fill: Some("count".to_string()),
                            },
                            options: HexBinOptionsCfg {
                                x: Some("bin-x".to_string()),
                                y: Some("bin-y".to_string()),
                                bin_width: Some(5),
                            },
                        })),
                    }),
                    MarkCfg::Line(LineOptionsCfg {
                        x: Some("lx".to_string()),
                        y: Some("ly".to_string()),
                        r: Some(1),
                        fill: None,
                        stroke: Some("line-stroke".to_string()),
                        transform: Some(TransformCfg::BinX(BinXTransformCfg {
                            outputs: TransformOutputsCfg {
                                x: None,
                                y: Some("bin-y-out".to_string()),
                                r: None,
                                z: None,
                                stroke: Some("bin-stroke".to_string()),
                                fill: None,
                            },
                            options: BinXOptionsCfg {
                                x: Some("binx-x".to_string()),
                                thresholds: Some(9),
                            },
                        })),
                    }),
                    MarkCfg::RectY(RectYOptionsCfg {
                        x0: Some("x0".to_string()),
                        x1: Some("x1".to_string()),
                        y0: Some("y0".to_string()),
                        y1: Some("y1".to_string()),
                        transform: None,
                    }),
                ],
                x: Some(AxisOptionsCfg {
                    label: Some("X Axis".to_string()),
                    anchor: Some("bottom".to_string()),
                    label_anchor: None,
                    label_arrow: Some("arrow".to_string()),
                }),
                y: Some(AxisOptionsCfg {
                    label: Some("Y Axis".to_string()),
                    anchor: Some("left".to_string()),
                    label_anchor: Some("center".to_string()),
                    label_arrow: None,
                }),
                margin: Some(10),
                margin_left: Some(11),
                margin_right: Some(15),
                margin_top: Some(12),
                margin_bottom: Some(13),
                inset: Some(8),
            }]),
            metrics: Some(vec![MetricCfg {
                label: "m1".to_string(),
                description: Some("desc".to_string()),
                unit_prefix: Some("prefix".to_string()),
                unit_suffix: Some("suffix".to_string()),
                value: "42".to_string(),
                precision: Some(2),
            }]),
        };

        let chart_without_optional = ChartCfg {
            document: default_document(),
            key: "chart-minimal".to_string(),
            scenario: Arc::new(ScenarioCfg {
                key: "scenario-b".to_string(),
                ..ScenarioCfg::default()
            }),
            plots: None,
            metrics: None,
        };

        let mut charts = HashMap::new();
        charts.insert(chart_with_optional.key.clone(), chart_with_optional.clone());
        charts.insert(
            chart_without_optional.key.clone(),
            chart_without_optional.clone(),
        );

        let yaml = ChartCfg::to_yaml_hash(&charts).unwrap();

        let mut expected = Hash::new();

        let mut chart_one = Hash::new();
        chart_one.insert(sy("scenario"), sy("scenario-a"));

        let mut chart_one_plots = Hash::new();
        let mut plot_hash = Hash::new();
        plot_hash.insert(sy("title"), sy("Plot Title"));
        plot_hash.insert(sy("subtitle"), sy("Plot Subtitle"));

        let mut plot_marks = Vec::new();
        let mut mark_hash = Hash::new();
        mark_hash.insert(sy("type"), sy("dot"));

        let mut mark_options = Hash::new();
        mark_options.insert(sy("x"), sy("x"));
        mark_options.insert(sy("y"), sy("y"));
        mark_options.insert(sy("r"), sy("7"));
        mark_options.insert(sy("fill"), sy("blue"));
        mark_options.insert(sy("stroke"), sy("red"));

        let mut transform_hash = Hash::new();
        transform_hash.insert(sy("type"), sy("hexbin"));

        let mut transform_content = Hash::new();
        let mut outputs_hash = Hash::new();
        outputs_hash.insert(sy("x"), sy("out-x"));
        outputs_hash.insert(sy("r"), sy("3"));
        outputs_hash.insert(sy("z"), sy("z-val"));
        outputs_hash.insert(sy("fill"), sy("count"));
        transform_content.insert(sy("outputs"), StrictYaml::Hash(outputs_hash));

        let mut options_hash = Hash::new();
        options_hash.insert(sy("x"), sy("bin-x"));
        options_hash.insert(sy("y"), sy("bin-y"));
        options_hash.insert(sy("bin-width"), sy("5"));
        transform_content.insert(sy("options"), StrictYaml::Hash(options_hash));

        transform_hash.insert(sy("content"), StrictYaml::Hash(transform_content));
        mark_options.insert(sy("transform"), StrictYaml::Hash(transform_hash));
        mark_hash.insert(sy("options"), StrictYaml::Hash(mark_options));
        plot_marks.push(StrictYaml::Hash(mark_hash));

        let mut line_mark_hash = Hash::new();
        line_mark_hash.insert(sy("type"), sy("line"));
        let mut line_options = Hash::new();
        line_options.insert(sy("x"), sy("lx"));
        line_options.insert(sy("y"), sy("ly"));
        line_options.insert(sy("r"), sy("1"));
        line_options.insert(sy("stroke"), sy("line-stroke"));

        let mut line_transform_hash = Hash::new();
        line_transform_hash.insert(sy("type"), sy("binx"));

        let mut line_transform_content = Hash::new();
        let mut line_outputs = Hash::new();
        line_outputs.insert(sy("y"), sy("bin-y-out"));
        line_outputs.insert(sy("stroke"), sy("bin-stroke"));
        line_transform_content.insert(sy("outputs"), StrictYaml::Hash(line_outputs));

        let mut line_options_hash = Hash::new();
        line_options_hash.insert(sy("x"), sy("binx-x"));
        line_options_hash.insert(sy("thresholds"), sy("9"));
        line_transform_content.insert(sy("options"), StrictYaml::Hash(line_options_hash));

        line_transform_hash.insert(sy("content"), StrictYaml::Hash(line_transform_content));
        line_options.insert(sy("transform"), StrictYaml::Hash(line_transform_hash));
        line_mark_hash.insert(sy("options"), StrictYaml::Hash(line_options));
        plot_marks.push(StrictYaml::Hash(line_mark_hash));

        let mut rect_mark_hash = Hash::new();
        rect_mark_hash.insert(sy("type"), sy("recty"));
        let mut rect_options = Hash::new();
        rect_options.insert(sy("x0"), sy("x0"));
        rect_options.insert(sy("x1"), sy("x1"));
        rect_options.insert(sy("y0"), sy("y0"));
        rect_options.insert(sy("y1"), sy("y1"));
        rect_mark_hash.insert(sy("options"), StrictYaml::Hash(rect_options));
        plot_marks.push(StrictYaml::Hash(rect_mark_hash));

        plot_hash.insert(sy("marks"), StrictYaml::Array(plot_marks));

        let mut x_axis = Hash::new();
        x_axis.insert(sy("label"), sy("X Axis"));
        x_axis.insert(sy("anchor"), sy("bottom"));
        x_axis.insert(sy("label-arrow"), sy("arrow"));
        plot_hash.insert(sy("x"), StrictYaml::Hash(x_axis));

        let mut y_axis = Hash::new();
        y_axis.insert(sy("label"), sy("Y Axis"));
        y_axis.insert(sy("anchor"), sy("left"));
        y_axis.insert(sy("label-anchor"), sy("center"));
        plot_hash.insert(sy("y"), StrictYaml::Hash(y_axis));

        plot_hash.insert(sy("margin"), sy("10"));
        plot_hash.insert(sy("margin-left"), sy("11"));
        plot_hash.insert(sy("margin-right"), sy("15"));
        plot_hash.insert(sy("margin-top"), sy("12"));
        plot_hash.insert(sy("margin-bottom"), sy("13"));
        plot_hash.insert(sy("inset"), sy("8"));

        chart_one_plots.insert(sy("plot1"), StrictYaml::Hash(plot_hash));
        chart_one.insert(sy("plots"), StrictYaml::Hash(chart_one_plots));

        let mut metric_hash = Hash::new();
        metric_hash.insert(sy("label"), sy("m1"));
        metric_hash.insert(sy("description"), sy("desc"));
        metric_hash.insert(sy("unit-prefix"), sy("prefix"));
        metric_hash.insert(sy("unit-suffix"), sy("suffix"));
        metric_hash.insert(sy("value"), sy("42"));
        metric_hash.insert(sy("precision"), sy("2"));
        chart_one.insert(
            sy("metrics"),
            StrictYaml::Array(vec![StrictYaml::Hash(metric_hash)]),
        );

        let mut chart_two = Hash::new();
        chart_two.insert(sy("scenario"), sy("scenario-b"));

        // Insert in deterministic (sorted) order to match serialization
        expected.insert(sy("chart-minimal"), StrictYaml::Hash(chart_two));
        expected.insert(sy("chart-with-options"), StrictYaml::Hash(chart_one));

        assert_eq!(yaml, StrictYaml::Hash(expected));
    }

    #[test]
    fn test_sanitize_drops_unknown_chart_keys() {
        let yaml = r#"
charts:
    test-chart:
        scenario: test
        plots: {}
        metrics: []
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let charts_section = root_hash
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let chart = charts_section
            .get(&StrictYaml::String("test-chart".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(chart.len(), 3);
        assert!(chart
            .get(&StrictYaml::String("scenario".to_string()))
            .is_some());
        assert!(chart
            .get(&StrictYaml::String("plots".to_string()))
            .is_some());
        assert!(chart
            .get(&StrictYaml::String("metrics".to_string()))
            .is_some());
        assert!(chart
            .get(&StrictYaml::String("unknown-key".to_string()))
            .is_none());
    }

    #[test]
    fn test_sanitize_drops_unknown_plot_keys() {
        let yaml = r#"
charts:
    test-chart:
        scenario: test
        plots:
            plot1:
                title: Test Plot
                marks: []
                unknown-key: should-drop
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let plot = doc_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("test-chart".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plots".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plot1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(plot.len(), 2);
        assert!(plot.get(&StrictYaml::String("title".to_string())).is_some());
        assert!(plot.get(&StrictYaml::String("marks".to_string())).is_some());
        assert!(plot
            .get(&StrictYaml::String("unknown-key".to_string()))
            .is_none());
    }

    #[test]
    fn test_sanitize_drops_unknown_mark_keys() {
        let yaml = r#"
charts:
    test-chart:
        scenario: test
        plots:
            plot1:
                marks:
                    - type: dot
                      options:
                          x: val
                      unknown-key: should-drop
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let marks = doc_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("test-chart".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plots".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plot1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("marks".to_string()))
            .unwrap()
            .as_vec()
            .unwrap();

        assert_eq!(marks.len(), 1);
        let mark = marks[0].as_hash().unwrap();
        assert_eq!(mark.len(), 2);
        assert!(mark.get(&StrictYaml::String("type".to_string())).is_some());
        assert!(mark
            .get(&StrictYaml::String("options".to_string()))
            .is_some());
        assert!(mark
            .get(&StrictYaml::String("unknown-key".to_string()))
            .is_none());
    }

    #[test]
    fn test_sanitize_drops_unknown_transform_keys() {
        let yaml = r#"
charts:
    test-chart:
        scenario: test
        plots:
            plot1:
                marks:
                    - type: dot
                      options:
                          transform:
                              type: hexbin
                              content:
                                  outputs:
                                      x: val
                                  options:
                                      x: val
                              unknown-key: should-drop
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let transform = doc_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("test-chart".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plots".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plot1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("marks".to_string()))
            .unwrap()
            .as_vec()
            .unwrap()[0]
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("options".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("transform".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(transform.len(), 2);
        assert!(transform
            .get(&StrictYaml::String("type".to_string()))
            .is_some());
        assert!(transform
            .get(&StrictYaml::String("content".to_string()))
            .is_some());
        assert!(transform
            .get(&StrictYaml::String("unknown-key".to_string()))
            .is_none());
    }

    #[test]
    fn test_sanitize_drops_unknown_metric_keys() {
        let yaml = r#"
charts:
    test-chart:
        scenario: test
        metrics:
            - label: test
              value: 42
              unknown-key: should-drop
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let metrics = doc_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("test-chart".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("metrics".to_string()))
            .unwrap()
            .as_vec()
            .unwrap();

        assert_eq!(metrics.len(), 1);
        let metric = metrics[0].as_hash().unwrap();
        assert_eq!(metric.len(), 2);
        assert!(metric
            .get(&StrictYaml::String("label".to_string()))
            .is_some());
        assert!(metric
            .get(&StrictYaml::String("value".to_string()))
            .is_some());
        assert!(metric
            .get(&StrictYaml::String("unknown-key".to_string()))
            .is_none());
    }

    #[test]
    fn test_sanitize_drops_unknown_axis_keys() {
        let yaml = r#"
charts:
    test-chart:
        scenario: test
        plots:
            plot1:
                marks: []
                x:
                    label: X Axis
                    anchor: bottom
                    unknown-key: should-drop
                y:
                    label: Y Axis
                    unknown-key: should-drop
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let plot = doc_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("test-chart".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plots".to_string()))
            .unwrap()
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("plot1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let x_axis = plot
            .get(&StrictYaml::String("x".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert_eq!(x_axis.len(), 2);
        assert!(x_axis
            .get(&StrictYaml::String("label".to_string()))
            .is_some());
        assert!(x_axis
            .get(&StrictYaml::String("anchor".to_string()))
            .is_some());

        let y_axis = plot
            .get(&StrictYaml::String("y".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert_eq!(y_axis.len(), 1);
        assert!(y_axis
            .get(&StrictYaml::String("label".to_string()))
            .is_some());
    }

    #[test]
    fn test_sanitize_lexicographic_ordering() {
        let yaml = r#"
charts:
    zebra:
        scenario: test
    alpha:
        scenario: test
    middle:
        scenario: test
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let charts = doc_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let keys: Vec<&str> = charts.iter().map(|(k, _)| k.as_str().unwrap()).collect();
        assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn test_sanitize_handles_missing_charts_section() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
"#;
        let doc = get_document(yaml);
        let result = ChartCfg::sanitize_documents(std::slice::from_ref(&doc));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_handles_non_hash_root() {
        let yaml = "just-a-string";
        let doc = get_document(yaml);
        let result = ChartCfg::sanitize_documents(std::slice::from_ref(&doc));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_skips_non_hash_charts_section() {
        let yaml = r#"
charts: just-a-string
"#;
        let doc = get_document(yaml);
        let result = ChartCfg::sanitize_documents(std::slice::from_ref(&doc));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_drops_non_hash_chart_entries() {
        let yaml = r#"
charts:
    valid-chart:
        scenario: test
    invalid-chart: just-a-string
"#;
        let doc = get_document(yaml);
        ChartCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let charts = doc_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("charts".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(charts.len(), 1);
        assert!(charts
            .get(&StrictYaml::String("valid-chart".to_string()))
            .is_some());
    }
}
