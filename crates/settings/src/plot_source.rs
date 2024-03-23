use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Plot {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub marks: Vec<Mark>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    Dot(DotMark),
    Line(LineMark),
    RectY(RectYMark),
    AxisX(AxisMark),
    AxisY(AxisMark),
}

// Dot mark
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DotMark {
    #[serde(flatten)]
    pub content: DotContent,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DotContent {
    #[serde(rename = "options")]
    Options(DotOptions),
    #[serde(rename = "transform")]
    Transform(Transform),
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DotOptions {
    pub x: Option<String>,
    pub y: Option<String>,
    pub r: Option<String>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
}

// Line mark
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LineMark {
    #[serde(flatten)]
    pub content: LineContent,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LineContent {
    #[serde(rename = "options")]
    Options(LineOptions),
    #[serde(rename = "transform")]
    Transform(Transform),
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LineOptions {
    pub x: Option<String>,
    pub y: Option<String>,
    pub r: Option<String>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
}

// RectY mark
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RectYMark {
    #[serde(flatten)]
    pub content: RectYContent,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RectYContent {
    #[serde(rename = "options")]
    Options(RectYOptions),
    #[serde(rename = "transform")]
    Transform(Transform),
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RectYOptions {
    pub x0: Option<String>,
    pub x1: Option<String>,
    pub y0: Option<String>,
    pub y1: Option<String>,
}

// AxisX mark
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AxisMark {
    pub label: Option<String>,
    pub anchor: Option<String>,
    pub label_anchor: Option<String>,
    pub label_arrow: Option<String>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Transform {
    HexBin(HexBinTransform),
    BinX(BinXTransform),
    // Other transform types can be added here
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TransformOutputs {
    x: Option<String>,
    y: Option<String>,
    r: Option<String>,
    z: Option<String>,
    stroke: Option<String>,
    fill: Option<String>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HexBinTransform {
    outputs: TransformOutputs,
    options: HexBinOptions,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HexBinOptions {
    x: Option<String>,
    y: Option<String>,
    #[serde(rename = "bin-width")]
    bin_width: Option<u32>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BinXTransform {
    outputs: TransformOutputs,
    options: BinXOptions,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BinXOptions {
    x: Option<String>,
    thresholds: Option<u32>,
}

impl TryFrom<String> for Plot {
    type Error = serde_yaml::Error;
    fn try_from(val: String) -> Result<Plot, Self::Error> {
        serde_yaml::from_str(&val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_yaml_into_dot_plot_source() {
        let yaml_data = r#"
title: Title
subtitle: Subtitle
marks:
-   type: Dot
    options:
        x: "0.1"
        y: "0.2"
        stroke: "black"
-   type: Dot
    transform:
        type: "HexBin"
        outputs:
            fill: "count"
        options:
            x: "0.1"
            y: "0.2"
            bin-width: 10"#
            .to_string();

        let plot: Plot = yaml_data.try_into().unwrap();

        assert_eq!(
            plot,
            Plot {
                title: Some("Title".to_string()),
                subtitle: Some("Subtitle".to_string()),
                marks: vec![
                    Mark::Dot(DotMark {
                        content: DotContent::Options(DotOptions {
                            r: None,
                            fill: None,
                            x: Some("0.1".to_string()),
                            y: Some("0.2".to_string()),
                            stroke: Some("black".to_string()),
                        }),
                    }),
                    Mark::Dot(DotMark {
                        content: DotContent::Transform(Transform::HexBin(HexBinTransform {
                            outputs: TransformOutputs {
                                x: None,
                                y: None,
                                r: None,
                                z: None,
                                stroke: None,
                                fill: Some("count".to_string()),
                            },
                            options: HexBinOptions {
                                x: Some("0.1".to_string()),
                                y: Some("0.2".to_string()),
                                bin_width: Some(10),
                            },
                        })),
                    })
                ]
            }
        );
    }

    #[test]
    fn parse_yaml_into_line_plot_source() {
        let yaml_data = r#"
title: Title
subtitle: Subtitle
marks:
-   type: Line
    options:
        x: "0.1"
        y: "0.2"
        stroke: "black""#
            .to_string();

        let plot: Plot = yaml_data.try_into().unwrap();

        assert_eq!(
            plot,
            Plot {
                title: Some("Title".to_string()),
                subtitle: Some("Subtitle".to_string()),
                marks: vec![Mark::Line(LineMark {
                    content: LineContent::Options(LineOptions {
                        r: None,
                        fill: None,
                        x: Some("0.1".to_string()),
                        y: Some("0.2".to_string()),
                        stroke: Some("black".to_string()),
                    }),
                }),],
            }
        );
    }

    #[test]
    fn parse_yaml_into_histogram() {
        let yaml_data = r#"
title: Title
subtitle: Subtitle
marks:
-   type: RectY
    transform:
        type: "BinX"
        outputs:
            y: "count"
        options:
            x: "0.1"
            thresholds: 10"#
            .to_string();

        let plot: Plot = yaml_data.try_into().unwrap();

        assert_eq!(
            plot,
            Plot {
                title: Some("Title".to_string()),
                subtitle: Some("Subtitle".to_string()),
                marks: vec![Mark::RectY(RectYMark {
                    content: RectYContent::Transform(Transform::BinX(BinXTransform {
                        outputs: TransformOutputs {
                            x: None,
                            y: Some("count".to_string()),
                            r: None,
                            z: None,
                            stroke: None,
                            fill: None,
                        },
                        options: BinXOptions {
                            x: Some("0.1".to_string()),
                            thresholds: Some(0),
                        },
                    })),
                }),],
            }
        );
    }
}
