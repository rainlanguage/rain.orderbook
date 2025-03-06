use super::try_extract_result_inner_type;
use crate::wasm_export::{UNCHECKED_RETURN_TYPE_PARAM, WASM_EXPORT_ATTR};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::ops::Deref;
use syn::{punctuated::Punctuated, Attribute, FnArg, ImplItemFn, Meta, Token, Type};

/// Collects function arguments and determines if the function has a self receiver
pub fn collect_function_arguments(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> (bool, Vec<TokenStream>) {
    let mut has_self_receiver = false;

    let args = inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => {
                has_self_receiver = true;
                None
            }
            FnArg::Typed(pat_type) => {
                let pat = pat_type.pat.deref();
                Some(quote! { #pat })
            }
        })
        .collect();

    (has_self_receiver, args)
}

/// Adds necessary attributes to the exported function
pub fn add_attributes_to_new_function(
    method: &mut ImplItemFn,
) -> Result<(Vec<Attribute>, Option<Type>), syn::Error> {
    // Forward the wasm_bindgen attributes to the new function
    let mut keep = Vec::new();
    let mut unchecked_ret_type: Option<String> = None;
    let mut wasm_bindgen_attrs: Vec<Attribute> = Vec::new();
    for attr in &method.attrs {
        if attr.path().is_ident(WASM_EXPORT_ATTR) {
            keep.push(false);
            let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for meta in nested {
                if meta.path().is_ident(UNCHECKED_RETURN_TYPE_PARAM) {
                    if unchecked_ret_type.is_some() {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "duplicate unchecked_return_type attribute",
                        ));
                    } else if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(str),
                        ..
                    }) = &meta.require_name_value()?.value
                    {
                        unchecked_ret_type = Some(str.value());
                    } else {
                        return Err(syn::Error::new_spanned(meta, "expected string literal"));
                    }
                } else {
                    // include unchanged
                    wasm_bindgen_attrs.push(syn::parse_quote!(
                        #[wasm_bindgen(#meta)]
                    ));
                }
            }
        } else {
            keep.push(true);
        }
    }

    let mut keep = keep.into_iter();
    method.attrs.retain(|_| keep.next().unwrap());

    // Create the modified return type and add the modified unchecked_return_type
    // Falls back to original return inner type if not provided by unchecked_return_type
    let inner_ret_type = try_extract_result_inner_type(method).cloned();
    if let Some(v) = unchecked_ret_type.or(inner_ret_type
        .as_ref()
        .map(|v| format!("{}", v.to_token_stream())))
    {
        let return_type = format!("WasmEncodedResult<{}>", v);
        wasm_bindgen_attrs.push(syn::parse_quote!(
            #[wasm_bindgen(unchecked_return_type = #return_type)]
        ));
    }

    Ok((wasm_bindgen_attrs, inner_ret_type))
}
