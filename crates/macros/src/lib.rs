use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, ImplItem, ItemImpl, Path, PathSegment, ReturnType, Type, TypePath,
};

const WASM_EXPORT_ATTR: &str = "wasm_export";
const SKIP_PARAM: &str = "skip";
const UNCHECKED_RETURN_TYPE_PARAM: &str = "unchecked_return_type";

#[proc_macro_attribute]
pub fn impl_wasm_exports(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as an impl block
    let mut input = parse_macro_input!(item as ItemImpl);

    // Transform each method to add a wasm export version
    let mut new_items = Vec::new();

    for item in input.items {
        if let ImplItem::Fn(mut method) = item {
            // Only process public functions that don't have skip_wasm_export attribute
            if let syn::Visibility::Public(_) = method.vis {
                let should_skip = should_skip_wasm_export(&method.attrs);

                if !should_skip {
                    if let ReturnType::Type(_, return_type) = &method.sig.output {
                        // Try to extract Result inner type (will skip non-Result functions)
                        if let Some(inner_type) = try_extract_result_inner_type(return_type) {
                            let fn_name = &method.sig.ident;
                            let is_async = method.sig.asyncness.is_some();
                            let args = collect_function_arguments(&method.sig.inputs);

                            // New function logic
                            {
                                let export_fn_name = syn::Ident::new(
                                    &format!("{}__{}", fn_name, WASM_EXPORT_ATTR),
                                    fn_name.span(),
                                );
                                let camel_case_name = to_camel_case(&fn_name.to_string());

                                // Create a new function with __wasm_export suffix
                                let mut export_method = method.clone();
                                export_method.sig.ident = export_fn_name;

                                add_attributes_to_new_function(
                                    &mut export_method,
                                    &camel_case_name,
                                );

                                let new_return_type = syn::parse_quote!(-> Result<CustomResult<#inner_type>, wasm_bindgen::JsValue>);
                                export_method.sig.output = new_return_type;

                                let call_expr = create_new_function_call(&fn_name, &args);

                                if is_async {
                                    export_method.block = syn::parse_quote!({
                                        let result: CustomResult<_> = #call_expr.await.into();
                                        Ok(result)
                                    });
                                } else {
                                    export_method.block = syn::parse_quote!({
                                        let result: CustomResult<_> = #call_expr.into();
                                        Ok(result)
                                    });
                                }

                                new_items.push(ImplItem::Fn(export_method));
                            }

                            // Add the skip_typescript attribute to the original method
                            method
                                .attrs
                                .push(syn::parse_quote!(#[wasm_bindgen(skip_typescript)]));
                        }
                    }
                }
            }

            // Keep the original item
            new_items.push(ImplItem::Fn(method));
        } else {
            // Keep the original item
            new_items.push(item.clone());
        }
    }

    input.items = new_items;

    // Generate the output with wasm_bindgen applied to the impl block
    let output = quote! {
        #[wasm_bindgen]
        #input
    };

    output.into()
}

#[proc_macro_attribute]
pub fn wasm_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simply return the item unchanged
    item
}

fn to_camel_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

// Try to extract the inner type from a Result<T, E> type, returning None if not a Result
fn try_extract_result_inner_type(return_type: &Box<Type>) -> Option<&Type> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = &**return_type
    {
        if let Some(PathSegment {
            ident, arguments, ..
        }) = segments.first()
        {
            if ident.to_string() == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = arguments {
                    if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                        return Some(t);
                    }
                }
            }
        }
    }
    None
}

fn collect_function_arguments(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Vec<proc_macro2::TokenStream> {
    inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                syn::FnArg::Receiver(receiver) => {
                    // Handle self parameter
                    if receiver.reference.is_some() {
                        if receiver.mutability.is_some() {
                            Some(quote::quote! { &mut self })
                        } else {
                            Some(quote::quote! { &self })
                        }
                    } else {
                        Some(quote::quote! { self })
                    }
                }
                syn::FnArg::Typed(pat_type) => {
                    // Handle named parameters
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        Some(quote::quote! { #pat_ident })
                    } else {
                        None
                    }
                }
            }
        })
        .collect()
}

fn add_attributes_to_new_function(method: &mut syn::ImplItemFn, camel_case_name: &str) {
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
                                if value.starts_with('"') && value.ends_with('"') {
                                    unchecked_value = &value[1..value.len() - 1];
                                } else if value.starts_with('\'') && value.ends_with('\'') {
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
                        format!("Promise<CustomResult<{}>>", unchecked_value)
                    } else {
                        format!("CustomResult<{}>", unchecked_value)
                    };

                    // Add other parameters
                    for param in &other_params {
                        // Parse the string parameter into a token stream
                        if let Ok(param_tokens) = syn::parse_str::<proc_macro2::TokenStream>(param)
                        {
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

    // Add the wasm_function attribute with the camelCase name
    // method
    //     .attrs
    //     .push(syn::parse_quote!(#[wasm_bindgen(js_name = #camel_case_name)]));

    // Extract the inner type from the Result return type
    // if let ReturnType::Type(_, return_type) = &method.sig.output {
    //     if let Some(inner_type) = try_extract_result_inner_type(return_type) {
    //         let ts_type = rust_type_to_ts_type(inner_type);

    //         // Check if the method is async and adjust the TypeScript return type accordingly
    //         let is_async = method.sig.asyncness.is_some();
    //         let return_type = if is_async {
    //             format!("Promise<CustomResult<{}>>", ts_type)
    //         } else {
    //             format!("CustomResult<{}>", ts_type)
    //         };

    //         method.attrs.push(syn::parse_quote!(
    //             #[wasm_bindgen(unchecked_return_type = #return_type)]
    //         ));
    //     }
    // }
}

/// Converts a Rust type to its TypeScript equivalent for wasm_bindgen
fn rust_type_to_ts_type(rust_type: &Type) -> String {
    match rust_type {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                // Handle primitive types
                match type_name.as_str() {
                    "String" | "str" => "string".to_string(),
                    "bool" => "boolean".to_string(),
                    "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "f32" | "f64" | "u64"
                    | "u128" | "i64" | "i128" => "number".to_string(),
                    "Vec" => {
                        // Handle Vec<T> -> Array<T>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                let inner_ts_type = rust_type_to_ts_type(inner_type);
                                return format!("Array<{}>", inner_ts_type);
                            }
                        }
                        "Array<any>".to_string()
                    }
                    "Option" => {
                        // Handle Option<T> -> T | null
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                let inner_ts_type = rust_type_to_ts_type(inner_type);
                                return format!("{} | undefined", inner_ts_type);
                            }
                        }
                        "any | undefined".to_string()
                    }
                    "HashMap" | "BTreeMap" => {
                        // Handle maps -> Record<K, V>
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if args.args.len() >= 2 {
                                if let (
                                    Some(syn::GenericArgument::Type(key_type)),
                                    Some(syn::GenericArgument::Type(value_type)),
                                ) = (args.args.first(), args.args.get(1))
                                {
                                    let key_ts_type = rust_type_to_ts_type(key_type);
                                    let value_ts_type = rust_type_to_ts_type(value_type);
                                    return format!("Map<{}, {}>", key_ts_type, value_ts_type);
                                }
                            }
                        }
                        "Map<any, any>".to_string()
                    }
                    // For custom types, use the type name directly
                    _ => type_name,
                }
            } else {
                "any".to_string()
            }
        }
        Type::Reference(type_ref) => {
            // Handle references like &str
            rust_type_to_ts_type(&type_ref.elem)
        }
        Type::Tuple(tuple) => {
            // Handle tuples -> [T, U, ...]
            if tuple.elems.is_empty() {
                "null".to_string()
            } else {
                let ts_types: Vec<String> = tuple
                    .elems
                    .iter()
                    .map(|elem| rust_type_to_ts_type(elem))
                    .collect();
                format!("[{}]", ts_types.join(", "))
            }
        }
        Type::Array(array) => {
            // Handle arrays -> Array<T>
            let inner_ts_type = rust_type_to_ts_type(&array.elem);
            format!("Array<{}>", inner_ts_type)
        }
        // For other types, default to "any"
        _ => "any".to_string(),
    }
}

fn create_new_function_call(
    fn_name: &syn::Ident,
    args: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    if let Some(first_arg) = args.first() {
        // Check if the first argument is self (indicating an instance method)
        if first_arg.to_string().contains("self") {
            if args.len() > 1 {
                quote::quote! { self.#fn_name(#(#args),*) }
            } else {
                quote::quote! { self.#fn_name() }
            }
        } else {
            // Static method call (no self)
            quote::quote! { Self::#fn_name(#(#args),*) }
        }
    } else {
        // No arguments at all, must be a static method
        quote::quote! { Self::#fn_name() }
    }
}

/// Checks if a method should skip WASM export generation
fn should_skip_wasm_export(attrs: &[syn::Attribute]) -> bool {
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
