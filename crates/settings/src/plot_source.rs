use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Plot {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub marks: Vec<Mark>,
    pub x: Option<AxisOptions>,
    pub y: Option<AxisOptions>,
    pub margin: Option<u32>,
    pub margin_left: Option<u32>,
    pub margin_right: Option<u32>,
    pub margin_top: Option<u32>,
    pub margin_bottom: Option<u32>,
    pub inset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[typeshare]
#[serde(tag = "type", content = "options")]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    Dot(DotOptions),
    Line(LineOptions),
    RectY(RectYOptions),
}
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DotOptions {
    pub x: Option<String>,
    pub y: Option<String>,
    pub r: Option<u32>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub transform: Option<Transform>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LineOptions {
    pub x: Option<String>,
    pub y: Option<String>,
    pub r: Option<u32>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub transform: Option<Transform>,
}

// RectY mark
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RectYOptions {
    pub x0: Option<String>,
    pub x1: Option<String>,
    pub y0: Option<String>,
    pub y1: Option<String>,
    pub transform: Option<Transform>,
}

// AxisX mark
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AxisOptions {
    pub label: Option<String>,
    pub anchor: Option<String>,
    pub label_anchor: Option<String>,
    pub label_arrow: Option<String>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "lowercase")]
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
    r: Option<u32>,
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
#[serde(rename_all = "kebab-case")]
pub struct HexBinOptions {
    x: Option<String>,
    y: Option<String>,
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
-   type: dot
    options:
        x: "0.1"
        y: "0.2"
        stroke: "black"
-   type: dot
    options:
        transform:
            type: hexbin
            content:
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
                x: None,
                y: None,
                margin: None,
                margin_left: None,
                margin_right: None,
                margin_top: None,
                margin_bottom: None,
                inset: None,
                marks: vec![
                    Mark::Dot(DotOptions {
                        r: None,
                        fill: None,
                        x: Some("0.1".to_string()),
                        y: Some("0.2".to_string()),
                        stroke: Some("black".to_string()),
                        transform: None,
                    },),
                    Mark::Dot(DotOptions {
                        r: None,
                        fill: None,
                        x: None,
                        y: None,
                        stroke: None,
                        transform: Some(Transform::HexBin(HexBinTransform {
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
                        }))
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
-   type: line
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
                x: None,
                y: None,
                margin: None,
                margin_left: None,
                margin_right: None,
                margin_top: None,
                margin_bottom: None,
                inset: None,
                marks: vec![Mark::Line(LineOptions {
                    transform: None,
                    r: None,
                    fill: None,
                    x: Some("0.1".to_string()),
                    y: Some("0.2".to_string()),
                    stroke: Some("black".to_string()),
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
-   type: recty
    options:
        transform:
            type: binx
            content:
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
                x: None,
                y: None,
                margin: None,
                margin_left: None,
                margin_right: None,
                margin_top: None,
                margin_bottom: None,
                inset: None,
                marks: vec![Mark::RectY(RectYOptions {
                    x0: None,
                    x1: None,
                    y0: None,
                    y1: None,
                    transform: Some(Transform::BinX(BinXTransform {
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
                            thresholds: Some(10),
                        },
                    })),
                }),],
            }
        );
    }
}
