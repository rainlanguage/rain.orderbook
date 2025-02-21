use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct PlotCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub marks: Vec<MarkCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<AxisOptionsCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<AxisOptionsCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inset: Option<u32>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(PlotCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "options")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub enum MarkCfg {
    Dot(DotOptionsCfg),
    Line(LineOptionsCfg),
    RectY(RectYOptionsCfg),
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(MarkCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DotOptionsCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DotOptionsCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct LineOptionsCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(LineOptionsCfg);

// RectY mark
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RectYOptionsCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RectYOptionsCfg);

// AxisX mark
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct AxisOptionsCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_anchor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_arrow: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(AxisOptionsCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub enum TransformCfg {
    HexBin(HexBinTransformCfg),
    BinX(BinXTransformCfg),
    // Other transform types can be added here
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TransformCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TransformOutputsCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    z: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fill: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TransformOutputsCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct HexBinTransformCfg {
    outputs: TransformOutputsCfg,
    options: HexBinOptionsCfg,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(HexBinTransformCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct HexBinOptionsCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bin_width: Option<u32>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(HexBinOptionsCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct BinXTransformCfg {
    outputs: TransformOutputsCfg,
    options: BinXOptionsCfg,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(BinXTransformCfg);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct BinXOptionsCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thresholds: Option<u32>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(BinXOptionsCfg);

impl TryFrom<String> for PlotCfg {
    type Error = serde_yaml::Error;
    fn try_from(val: String) -> Result<PlotCfg, Self::Error> {
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

        let plot: PlotCfg = yaml_data.try_into().unwrap();

        assert_eq!(
            plot,
            PlotCfg {
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
                    MarkCfg::Dot(DotOptionsCfg {
                        r: None,
                        fill: None,
                        x: Some("0.1".to_string()),
                        y: Some("0.2".to_string()),
                        stroke: Some("black".to_string()),
                        transform: None,
                    },),
                    MarkCfg::Dot(DotOptionsCfg {
                        r: None,
                        fill: None,
                        x: None,
                        y: None,
                        stroke: None,
                        transform: Some(TransformCfg::HexBin(HexBinTransformCfg {
                            outputs: TransformOutputsCfg {
                                x: None,
                                y: None,
                                r: None,
                                z: None,
                                stroke: None,
                                fill: Some("count".to_string()),
                            },
                            options: HexBinOptionsCfg {
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

        let plot: PlotCfg = yaml_data.try_into().unwrap();

        assert_eq!(
            plot,
            PlotCfg {
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
                marks: vec![MarkCfg::Line(LineOptionsCfg {
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

        let plot: PlotCfg = yaml_data.try_into().unwrap();

        assert_eq!(
            plot,
            PlotCfg {
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
                marks: vec![MarkCfg::RectY(RectYOptionsCfg {
                    x0: None,
                    x1: None,
                    y0: None,
                    y1: None,
                    transform: Some(TransformCfg::BinX(BinXTransformCfg {
                        outputs: TransformOutputsCfg {
                            x: None,
                            y: Some("count".to_string()),
                            r: None,
                            z: None,
                            stroke: None,
                            fill: None,
                        },
                        options: BinXOptionsCfg {
                            x: Some("0.1".to_string()),
                            thresholds: Some(10),
                        },
                    })),
                }),],
            }
        );
    }
}
