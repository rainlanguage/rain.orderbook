#[cfg(target_family = "wasm")]
pub mod bindings;
#[cfg(any(target_family = "wasm", test))]
pub mod common;
#[cfg(target_family = "wasm")]
pub mod config;
#[cfg(target_family = "wasm")]
pub mod gui;
#[cfg(any(target_family = "wasm", test))]
pub mod subgraph;
#[cfg(target_family = "wasm")]
pub mod yaml;

// re-export other crates to include their wasm bindings as single export point
#[cfg(target_family = "wasm")]
pub use rain_orderbook_app_settings;
#[cfg(target_family = "wasm")]
pub use rain_orderbook_common;
#[cfg(target_family = "wasm")]
pub use rain_orderbook_quote;
#[cfg(target_family = "wasm")]
pub use rain_orderbook_subgraph_client;

use wasm_bindgen_utils::prelude::wasm_bindgen;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type Address = `0x${string}`;
export type Hex = `0x${string}`;
"#;
