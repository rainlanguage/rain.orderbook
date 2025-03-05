// Module for WASM export functionality
mod attributes;
mod function_gen;
mod transformers;
mod type_utils;

// Re-export the public API
pub use attributes::should_skip_wasm_export;
pub use function_gen::create_new_function_call;
pub use transformers::{add_attributes_to_new_function, collect_function_arguments};
pub use type_utils::try_extract_result_inner_type;

// Constants used throughout the module
pub const WASM_EXPORT_ATTR: &str = "wasm_export";
pub const SKIP_PARAM: &str = "skip";
pub const UNCHECKED_RETURN_TYPE_PARAM: &str = "unchecked_return_type";
