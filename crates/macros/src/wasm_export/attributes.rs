use crate::wasm_export::{SKIP_PARAM, WASM_EXPORT_ATTR};
use syn::Attribute;

/// Checks if a method should skip WASM export generation
pub fn should_skip_wasm_export(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident(WASM_EXPORT_ATTR) {
            if let Ok(meta) = attr.meta.require_list() {
                if let Ok(nested) = meta.parse_args::<syn::Meta>() {
                    return nested.path().is_ident(SKIP_PARAM);
                }
            }
            false
        } else {
            false
        }
    })
}
