pub use rain_interpreter_eval::trace::*;
use rain_orderbook_app_settings::chart::ChartCfg;
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
