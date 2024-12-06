use super::*;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TransformOutputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HexBinTransform {
    pub outputs: TransformOutputs,
    pub options: HexBinOptions,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct HexBinOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_width: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BinXOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thresholds: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BinXTransform {
    pub outputs: TransformOutputs,
    pub options: BinXOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "options")]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    Dot(DotOptions),
    Line(LineOptions),
    RectY(RectYOptions),
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DotOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LineOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RectYOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "lowercase")]
pub enum Transform {
    HexBin(HexBinTransform),
    BinX(BinXTransform),
    // Other transform types can be added here
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AxisOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_anchor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_arrow: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PlotYaml {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub marks: Vec<Mark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<AxisOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<AxisOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inset: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plots: Option<HashMap<String, PlotYaml>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<MetricYaml>>,
}
impl ChartYaml {
    fn parse_marks(plot_key: &str, plot_value: &StrictYaml) -> Result<Vec<Mark>, YamlError> {
        let marks_vector: &Vec<StrictYaml> = require_vec(
            plot_value,
            "marks",
            Some(format!("marks vector missing for plot: {}", plot_key)),
        )?;
        let mut marks: Vec<Mark> = vec![];
        for (i, mark) in marks_vector.iter().enumerate() {
            let mark = require_hash(
                mark,
                None,
                Some(format!(
                    "each mark value must be a map for index: {} for plot: {}",
                    i, plot_key
                )),
            )?;
            let mark_options = get_hash_value(
                mark,
                "options",
                Some(format!(
                    "options field missing for mark index: {} for plot: {}",
                    i, plot_key
                )),
            )?;
            let mark_type = require_string(
                get_hash_value(
                    mark,
                    "type",
                    Some(format!(
                        "type field missing for mark index: {} for plot: {}",
                        i, plot_key
                    )),
                )?,
                None,
                Some(format!(
                    "type field must be string for mark index: {} for plot: {}",
                    i, plot_key
                )),
            )?;
            let mark = match mark_type.as_str() {
                "dot" => {
                    let x = optional_string(mark_options, "x");
                    let y = optional_string(mark_options, "y");
                    let r = optional_string(mark_options, "r");
                    let fill = optional_string(mark_options, "fill");
                    let stroke = optional_string(mark_options, "stroke");
                    let transform = optional_hash(mark_options, "transform")
                        .map(|transform| Self::parse_transform(plot_key, i, transform))
                        .transpose()?;
                    Mark::Dot(DotOptions {
                        x,
                        y,
                        r,
                        fill,
                        stroke,
                        transform,
                    })
                }
                "line" => {
                    let x = optional_string(mark_options, "x");
                    let y = optional_string(mark_options, "y");
                    let r = optional_string(mark_options, "r");
                    let fill = optional_string(mark_options, "fill");
                    let stroke = optional_string(mark_options, "stroke");
                    let transform = optional_hash(mark_options, "transform")
                        .map(|transform| Self::parse_transform(plot_key, i, transform))
                        .transpose()?;
                    Mark::Line(LineOptions {
                        x,
                        y,
                        r,
                        fill,
                        stroke,
                        transform,
                    })
                }
                "recty" => {
                    let x0 = optional_string(mark_options, "x0");
                    let x1 = optional_string(mark_options, "x1");
                    let y0 = optional_string(mark_options, "y0");
                    let y1 = optional_string(mark_options, "y1");
                    let transform = optional_hash(mark_options, "transform")
                        .map(|transform| Self::parse_transform(plot_key, i, transform))
                        .transpose()?;
                    Mark::RectY(RectYOptions {
                        x0,
                        x1,
                        y0,
                        y1,
                        transform,
                    })
                }
                _ => {
                    return Err(YamlError::ParseError(format!(
                        "invalid mark type: {} for mark index: {} for plot: {}",
                        mark_type, i, plot_key
                    )))
                }
            };
            marks.push(mark);
        }
        Ok(marks)
    }

    fn parse_transform(
        plot_key: &str,
        mark_index: usize,
        transform: &LinkedHashMap<StrictYaml, StrictYaml>,
    ) -> Result<Transform, YamlError> {
        let transform_type = require_string(
            get_hash_value(
                transform,
                "type",
                Some(format!(
                    "type field missing for transform for mark index: {} for plot: {}",
                    mark_index, plot_key
                )),
            )?,
            None,
            Some(format!(
                "type field must be a string for transform for mark index: {} for plot: {}",
                mark_index, plot_key
            )),
        )?;
        let transform_content = get_hash_value(
            transform,
            "content",
            Some(format!(
                "content missing for transform for mark index: {} for plot: {}",
                mark_index, plot_key
            )),
        )?;
        // TODO: Since all the fields of outputs are optional
        // maybe outputs should also be optional
        let outputs = require_hash(
            transform_content,
            Some("outputs"),
            Some(format!(
                "outputs missing for transform content for mark index: {} for plot: {}",
                mark_index, plot_key
            )),
        )?;
        let x = get_hash_value_as_option(outputs, "x")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "x must be string for transform for mark index: {} for plot: {}",
                        mark_index, plot_key
                    )),
                )
            })
            .transpose()?;
        let y = get_hash_value_as_option(outputs, "y")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "y must be string for transform for mark index: {} for plot: {}",
                        mark_index, plot_key
                    )),
                )
            })
            .transpose()?;
        let r = get_hash_value_as_option(outputs, "r")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "r must be string for transform for mark index: {} for plot: {}",
                        mark_index, plot_key
                    )),
                )
            })
            .transpose()?;
        let z = get_hash_value_as_option(outputs, "z")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "z must be string for transform for mark index: {} for plot: {}",
                        mark_index, plot_key
                    )),
                )
            })
            .transpose()?;
        let stroke = get_hash_value_as_option(outputs, "stroke")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "stroke must be string for transform for mark index: {} for plot: {}",
                        mark_index, plot_key
                    )),
                )
            })
            .transpose()?;
        let fill = get_hash_value_as_option(outputs, "fill")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "fill must be string for transform for mark index: {} for plot: {}",
                        mark_index, plot_key
                    )),
                )
            })
            .transpose()?;
        let transform_outputs = TransformOutputs {
            x,
            y,
            r,
            z,
            stroke,
            fill,
        };

        // TODO: Since all the fields of options are optional
        // maybe options should also be optional
        let options = require_hash(
            transform_content,
            Some("options"),
            Some(format!(
                "options missing for transform content for mark index: {} for plot: {}",
                mark_index, plot_key
            )),
        )?;
        match transform_type.as_str() {
            "hexbin" => {
                let x = get_hash_value_as_option(options, "x")
                    .map(|v| {
                        require_string(
                            v,
                            None,
                            Some(format!(
                                "x must be string for transform for mark index: {} for plot: {}",
                                mark_index, plot_key
                            )),
                        )
                    })
                    .transpose()?;
                let y = get_hash_value_as_option(options, "y")
                    .map(|v| {
                        require_string(
                            v,
                            None,
                            Some(format!(
                                "y must be string for transform for mark index: {} for plot: {}",
                                mark_index, plot_key
                            )),
                        )
                    })
                    .transpose()?;
                let bin_width = get_hash_value_as_option(options, "bin_width")
                    .map(|v| {
                        require_string(
                            v,
                            None,
                            Some(format!(
                            "bin_width must be string for transform for mark index: {} for plot: {}",
                            mark_index, plot_key
                        )),
                        )
                    })
                    .transpose()?;
                Ok(Transform::HexBin(HexBinTransform {
                    outputs: transform_outputs,
                    options: HexBinOptions { x, y, bin_width },
                }))
            }
            "binx" => {
                let x = get_hash_value_as_option(options, "x")
                    .map(|v| {
                        require_string(
                            v,
                            None,
                            Some(format!(
                                "x must be string for transform for mark index: {} for plot: {}",
                                mark_index, plot_key
                            )),
                        )
                    })
                    .transpose()?;
                let thresholds = get_hash_value_as_option(options, "thresholds")
                    .map(|v| {
                        require_string(
                            v,
                            None,
                            Some(format!(
                            "thresholds must be string for transform for mark index: {} for plot: {}",
                            mark_index, plot_key
                        )),
                        )
                    })
                    .transpose()?;
                Ok(Transform::BinX(BinXTransform {
                    outputs: transform_outputs,
                    options: BinXOptions { x, thresholds },
                }))
            }
            _ => {
                return Err(YamlError::ParseError(format!(
                    "invalid transform type: {} for mark index: {} for plot: {}",
                    transform_type, mark_index, plot_key
                )));
            }
        }
    }

    fn parse_axis_options(
        plot_key: &str,
        axis_key: &str,
        value: &LinkedHashMap<StrictYaml, StrictYaml>,
    ) -> Result<AxisOptions, YamlError> {
        let label = get_hash_value_as_option(value, "label")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "label field must be string for axis options: {} for plot: {}",
                        axis_key, plot_key
                    )),
                )
            })
            .transpose()?;
        let anchor = get_hash_value_as_option(value, "anchor")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "anchor field must be string for axis options: {} for plot: {}",
                        axis_key, plot_key
                    )),
                )
            })
            .transpose()?;
        let label_anchor = get_hash_value_as_option(value, "label_anchor")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "label_anchor field must be string for axis options: {} for plot: {}",
                        axis_key, plot_key
                    )),
                )
            })
            .transpose()?;
        let label_arrow = get_hash_value_as_option(value, "label_arrow")
            .map(|v| {
                require_string(
                    v,
                    None,
                    Some(format!(
                        "label_arrow field must be string for axis options: {} for plot: {}",
                        axis_key, plot_key
                    )),
                )
            })
            .transpose()?;
        Ok(AxisOptions {
            label,
            anchor,
            label_anchor,
            label_arrow,
        })
    }

    pub fn try_from_string(source: &str) -> Result<HashMap<String, Self>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut charts = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("charts"),
            Some("missing field: charts".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let mut chart = Self {
                scenario: optional_string(value, "scenario"),
                plots: None,
                metrics: None,
            };

            if let Some(plots) = optional_hash(value, "plots") {
                let mut plots_map = HashMap::new();
                for (plot_key, plot_value) in plots {
                    let plot_key = plot_key.as_str().unwrap_or_default();

                    let title = require_string(
                        plot_value,
                        Some("title"),
                        Some(format!("title missing for plot: {}", plot_key)),
                    )?;
                    let subtitle = optional_string(plot_value, "subtitle");
                    let marks = Self::parse_marks(plot_key, plot_value)?;
                    let x = optional_hash(plot_value, "x")
                        .map(|v| Self::parse_axis_options(plot_key, "x", v))
                        .transpose()?;
                    let y = optional_hash(plot_value, "y")
                        .map(|v| Self::parse_axis_options(plot_key, "y", v))
                        .transpose()?;
                    let margin = optional_string(plot_value, "margin");
                    let margin_left = optional_string(plot_value, "margin_left");
                    let margin_right = optional_string(plot_value, "margin_right");
                    let margin_top = optional_string(plot_value, "margin_top");
                    let margin_bottom = optional_string(plot_value, "margin_bottom");
                    let inset = optional_string(plot_value, "inset");

                    plots_map.insert(
                        plot_key.to_string(),
                        PlotYaml {
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
                        },
                    );
                }
                chart.plots = Some(plots_map);
            }

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
            charts.insert(key.to_string(), chart);
        }
        Ok(charts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charts_errors() {
        let yaml = r#"
test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: charts".to_string())
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("title missing for plot: plot1".to_string())
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("marks vector missing for plot: plot1".to_string())
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                  - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "each mark value must be a map for index: 0 for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - test: test

        "#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "options field missing for mark index: 0 for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - test: test
                      options:

"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "type field missing for mark index: 0 for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type:
                        - test
                      options:

"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "type field must be string for mark index: 0 for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type:
                        test: test
                      options:

"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "type field must be string for mark index: 0 for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: test
                      options:
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "invalid mark type: test for mark index: 0 for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                        transform:
                            test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "type field missing for transform for mark index: 0 for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                        transform:
                            type:
                                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "type field must be a string for transform for mark index: 0 for plot: plot1"
                    .to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                        transform:
                            type:
                                - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "type field must be a string for transform for mark index: 0 for plot: plot1"
                    .to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                        transform:
                            type: test
                            content:
                                outputs:
                                    test: test
                                options:
                                    test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "invalid transform type: test for mark index: 0 for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    label:
                        - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label field must be string for axis options: x for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    label:
                        - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label field must be string for axis options: x for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    anchor:
                        - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "anchor field must be string for axis options: x for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    anchor:
                        - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "anchor field must be string for axis options: x for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    label_anchor:
                        - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label_anchor field must be string for axis options: x for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    label_anchor:
                        - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label_anchor field must be string for axis options: x for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    label_arrow:
                        - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label_arrow field must be string for axis options: x for plot: plot1".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        plots:
            plot1:
                title: this is a title
                marks:
                    - type: dot
                      options:
                x:
                    label_arrow:
                        - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label_arrow field must be string for axis options: x for plot: plot1".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        metrics:
            - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metric value must be a map for index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        metrics:
            - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label missing for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label:
                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label:
                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "label must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description:
                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "description must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description:
                - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "description must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix:
                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_prefix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix:
                - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_prefix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix:
                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_suffix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix:
                - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "unit_suffix must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "value missing for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value:
                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "value must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value:
                - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "value must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );

        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value: test
              precision:
                - test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "precision must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
        let yaml = r#"
charts:
    chart1:
        metrics:
            - label: test
              description: test
              unit_prefix: test
              unit_suffix: test
              value: test
              precision:
                - test: test
"#;
        let error = ChartYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "precision must be string for metric index 0 in chart \"chart1\"".to_string()
            )
        );
    }
}
