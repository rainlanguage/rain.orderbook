use crate::wasm_export::{UNCHECKED_RETURN_TYPE_PARAM, WASM_EXPORT_ATTR};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, FnArg, ImplItemFn};

/// Collects function arguments and determines if the function has a self receiver
pub fn collect_function_arguments(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> (bool, Vec<TokenStream>) {
    let mut has_self_receiver = false;

    let args = inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Receiver(_) => {
                    has_self_receiver = true;
                    None
                }
                FnArg::Typed(pat_type) => {
                    // Extract the pattern (variable name)
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        Some(quote! { #pat_ident })
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    (has_self_receiver, args)
}

/// Adds necessary attributes to the exported function
pub fn add_attributes_to_new_function(method: &mut ImplItemFn) -> Vec<Attribute> {
    // Forward the wasm_bindgen attributes to the new function
    let mut wasm_bindgen_attrs: Vec<Attribute> = Vec::new();
    let mut keep = Vec::new();
    for attr in &method.attrs {
        if attr.path().is_ident(WASM_EXPORT_ATTR) {
            keep.push(false);
            let mut unchecked_ret_type = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(UNCHECKED_RETURN_TYPE_PARAM) {
                    if let syn::Meta::NameValue(syn::MetaNameValue {
                        value:
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(str),
                                ..
                            }),
                        ..
                    }) = &attr.meta
                    {
                        unchecked_ret_type = Some(str.value());
                    }
                } else {
                    // Forward other attributes unchanged
                    if let Some(v) = meta.path.get_ident() {
                        let mut tokens = v.to_string();
                        tokens.push_str(&meta.input.to_string());
                        if let Ok(param_tokens) = syn::parse_str::<TokenStream>(&tokens) {
                            wasm_bindgen_attrs.push(syn::parse_quote!(
                                #[wasm_bindgen(#param_tokens)]
                            ));
                        }
                    }
                }
                Ok(())
            });
            if let Some(v) = unchecked_ret_type {
                // Create the modified return type
                let return_type = format!("WasmEncodedResult<{}>", v);
                // Add the modified unchecked_return_type
                wasm_bindgen_attrs.push(syn::parse_quote!(
                    #[wasm_bindgen(unchecked_return_type = #return_type)]
                ));
            }
        } else {
            keep.push(true);
        }
    }

    let mut keep = keep.into_iter();
    method.attrs.retain(|_attr| keep.next().unwrap());

    wasm_bindgen_attrs
}
