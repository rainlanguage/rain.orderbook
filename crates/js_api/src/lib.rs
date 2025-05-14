#[cfg(target_family = "wasm")]
pub mod bindings;
#[cfg(any(target_family = "wasm", test))]
pub mod common;
#[cfg(target_family = "wasm")]
pub mod gui;
#[cfg(target_family = "wasm")]
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
