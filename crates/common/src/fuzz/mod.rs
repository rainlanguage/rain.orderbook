use alloy::primitives::{Address, U256};
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use rain_interpreter_bindings::{
    DeployerISP::{iInterpreterCall, iStoreCall},
    IInterpreterV3::eval3Call,
};
pub use rain_interpreter_eval::trace::*;
use rain_orderbook_app_settings::{
    chart::ChartCfg,
    order::OrderIOCfg,
    yaml::{dotrain::DotrainYaml, YamlError, YamlParsable},
};
use rain_orderbook_bindings::IERC20;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, serialize_hashmap_as_object};

#[cfg(not(target_family = "wasm"))]
mod impls;
#[cfg(not(target_family = "wasm"))]
pub use impls::*;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ChartData {
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, FuzzResultFlat>")
    )]
    scenarios_data: HashMap<String, FuzzResultFlat>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ChartCfg>")
    )]
    charts: HashMap<String, ChartCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ChartData);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FuzzResultFlat {
    pub scenario: String,
    pub data: RainEvalResultsTable,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(FuzzResultFlat);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DeploymentDebugData {
    pub result: HashMap<String, Vec<DeploymentDebugPairData>>,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub block_number: U256,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentDebugData);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DeploymentDebugPairData {
    pub order: String,
    pub scenario: String,
    pub pair: String,
    pub result: Option<FuzzResultFlat>,
    pub error: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentDebugPairData);
