use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemImpl, ReturnType};

// Import modules
mod wasm_export;

// Import specific items from modules
use wasm_export::{
    add_attributes_to_new_function, collect_function_arguments, create_new_function_call,
    should_skip_wasm_export, try_extract_result_inner_type, WASM_EXPORT_ATTR,
};

#[proc_macro_attribute]
pub fn impl_wasm_exports(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as an impl block
    let input = parse_macro_input!(item as ItemImpl);

    // Create two vectors to store original and exported items
    let mut original_items = Vec::new();
    let mut export_items = Vec::new();

    for item in input.items.iter() {
        if let ImplItem::Fn(method) = item {
            // Add original method to original_items
            original_items.push(ImplItem::Fn(method.clone()));

            // Process for export if applicable
            if let syn::Visibility::Public(_) = method.vis {
                let should_skip = should_skip_wasm_export(&method.attrs);

                if !should_skip {
                    if let ReturnType::Type(_, return_type) = &method.sig.output {
                        if let Some(inner_type) = try_extract_result_inner_type(return_type) {
                            let fn_name = &method.sig.ident;
                            let is_async = method.sig.asyncness.is_some();
                            let (has_self_receiver, args) =
                                collect_function_arguments(&method.sig.inputs);

                            // Create exported version
                            let export_fn_name = syn::Ident::new(
                                &format!("{}__{}", fn_name, WASM_EXPORT_ATTR),
                                fn_name.span(),
                            );

                            let mut export_method = method.clone();
                            export_method.sig.ident = export_fn_name;

                            add_attributes_to_new_function(&mut export_method);

                            let new_return_type = syn::parse_quote!(-> Result<WasmEncodedResult<#inner_type>, wasm_bindgen::JsValue>);
                            export_method.sig.output = new_return_type;

                            let call_expr =
                                create_new_function_call(fn_name, has_self_receiver, &args);

                            if is_async {
                                export_method.block = syn::parse_quote!({
                                    let result: WasmEncodedResult<_> = #call_expr.await.into();
                                    Ok(result)
                                });
                            } else {
                                export_method.block = syn::parse_quote!({
                                    let result: WasmEncodedResult<_> = #call_expr.into();
                                    Ok(result)
                                });
                            }

                            export_items.push(ImplItem::Fn(export_method));
                        }
                    }
                }
            }
        } else {
            // Non-function items go to both impl blocks
            original_items.push(item.clone());
        }
    }

    // Create two impl blocks
    let mut original_impl = input.clone();
    original_impl.items = original_items;

    let mut export_impl = input;
    export_impl.items = export_items;

    // Generate the output with wasm_bindgen only on the export impl
    let output = quote! {
        #original_impl

        #[wasm_bindgen]
        #export_impl
    };

    output.into()
}

#[proc_macro_attribute]
pub fn wasm_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simply return the item unchanged
    item
}
