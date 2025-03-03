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
pub fn add_attributes_to_new_function(method: &mut ImplItemFn) {
    // Add the allow attribute to suppress the warning
    method
        .attrs
        .push(syn::parse_quote!(#[allow(non_snake_case)]));

    // Forward the wasm_bindgen attributes to the new function
    let mut wasm_bindgen_attrs: Vec<Attribute> = Vec::new();
    for attr in &method.attrs {
        if attr.path().is_ident(WASM_EXPORT_ATTR) {
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();

                // Check if this attribute contains unchecked_return_type
                if tokens.contains(UNCHECKED_RETURN_TYPE_PARAM) {
                    // Extract the value from unchecked_return_type
                    let mut unchecked_value = "";
                    let mut other_params = Vec::new();

                    // Parse the tokens to extract individual parameters
                    for param in tokens.split(',') {
                        let param = param.trim();
                        if param.starts_with(UNCHECKED_RETURN_TYPE_PARAM) {
                            // Extract the value between quotes
                            if let Some(value) = param.split('=').nth(1) {
                                let value = value.trim();
                                // Remove quotes if present
                                if (value.starts_with('"') && value.ends_with('"'))
                                    || (value.starts_with('\'') && value.ends_with('\''))
                                {
                                    unchecked_value = &value[1..value.len() - 1];
                                } else {
                                    unchecked_value = value;
                                }
                            }
                        } else {
                            // Keep other parameters
                            other_params.push(param.to_string());
                        }
                    }

                    // Create the modified return type
                    let return_type = if method.sig.asyncness.is_some() {
                        format!("Promise<WasmEncodedResult<{}>>", unchecked_value)
                    } else {
                        format!("WasmEncodedResult<{}>", unchecked_value)
                    };

                    // Add other parameters
                    for param in &other_params {
                        // Parse the string parameter into a token stream
                        if let Ok(param_tokens) = syn::parse_str::<TokenStream>(param) {
                            wasm_bindgen_attrs.push(syn::parse_quote!(
                                #[wasm_bindgen(#param_tokens)]
                            ));
                        }
                    }
                    // Add the modified unchecked_return_type
                    wasm_bindgen_attrs.push(syn::parse_quote!(
                        #[wasm_bindgen(unchecked_return_type = #return_type)]
                    ));
                } else {
                    // Forward other attributes unchanged
                    let tokens = meta.tokens.clone();
                    wasm_bindgen_attrs.push(syn::parse_quote!(#[wasm_bindgen(#tokens)]));
                }
            }
        }
    }

    if !wasm_bindgen_attrs.is_empty() {
        method.attrs.extend(wasm_bindgen_attrs);
    }
}
