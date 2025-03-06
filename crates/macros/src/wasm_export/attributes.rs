use crate::wasm_export::{SKIP_PARAM, WASM_EXPORT_ATTR};
use syn::Attribute;

/// Checks if a method should skip WASM export generation
pub fn should_skip_wasm_export(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident(WASM_EXPORT_ATTR) {
            let mut skip = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(SKIP_PARAM) {
                    skip = true;
                }
                Ok(())
            });
            skip
        } else {
            false
        }
    })
}
