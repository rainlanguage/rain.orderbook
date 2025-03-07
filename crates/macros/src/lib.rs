use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ImplItem, ItemImpl};

// Import modules
mod wasm_export;

// Import specific items from modules
use wasm_export::{
    collect_function_arguments, create_new_function_call, handle_attrs, WASM_EXPORT_ATTR,
};

#[proc_macro_attribute]
pub fn wasm_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_macro(attr, item) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand_macro(_attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    // Parse the input as an impl block
    let mut input = syn::parse::<ItemImpl>(item)?;

    // Create vector to store exported items
    let mut export_items = Vec::new();

    for item in input.items.iter_mut() {
        if let ImplItem::Fn(method) = item {
            // Process for export if applicable
            if let syn::Visibility::Public(_) = method.vis {
                let (forwarding_attrs, inner_ret_type, should_skip) = handle_attrs(method)?;
                if should_skip {
                    continue;
                }
                if let Some(inner_ret_type) = inner_ret_type {
                    let fn_name = &method.sig.ident;
                    let is_async = method.sig.asyncness.is_some();
                    let (has_self_receiver, args) = collect_function_arguments(&method.sig.inputs);

                    // Create exported version
                    let export_fn_name = syn::Ident::new(
                        &format!("{}__{}", fn_name, WASM_EXPORT_ATTR),
                        fn_name.span(),
                    );

                    let mut export_method = method.clone();
                    export_method.sig.ident = export_fn_name;
                    export_method
                        .attrs
                        .push(syn::parse_quote!(#[allow(non_snake_case)]));
                    export_method.attrs.extend(forwarding_attrs);

                    let new_return_type = syn::parse_quote!(-> WasmEncodedResult<#inner_ret_type>);
                    export_method.sig.output = new_return_type;

                    let call_expr = create_new_function_call(fn_name, has_self_receiver, &args);

                    if is_async {
                        export_method.block = syn::parse_quote!({
                            #call_expr.await.into()
                        });
                    } else {
                        export_method.block = syn::parse_quote!({
                            #call_expr.into()
                        });
                    }

                    export_items.push(ImplItem::Fn(export_method));
                } else {
                    return Err(Error::new_spanned(
                        &method.sig.output,
                        "wasm_macro expects Result<T, E> return type",
                    ));
                }
            }
        }
    }

    // Create two impl blocks
    let original_impl = input.clone();

    let mut export_impl = input;
    export_impl.items = export_items;

    // Generate the output with wasm_bindgen only on the export impl
    let output = quote! {
        #original_impl

        #[wasm_bindgen]
        #export_impl
    };

    Ok(output.into())
}
