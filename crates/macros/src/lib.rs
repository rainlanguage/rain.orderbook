use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemImpl};

#[proc_macro_attribute]
pub fn print_fn_names(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as an impl block
    let mut input = parse_macro_input!(item as ItemImpl);

    // Transform each method to add the wasm_function attribute
    input.items = input
        .items
        .into_iter()
        .map(|item| {
            if let ImplItem::Fn(mut method) = item {
                // Only process public functions
                if let syn::Visibility::Public(_) = method.vis {
                    let fn_name = &method.sig.ident;
                    let camel_case_name = to_camel_case(&fn_name.to_string());

                    // Add the wasm_function attribute with the camelCase name
                    method
                        .attrs
                        .push(syn::parse_quote!(#[wasm_bindgen(js_name = #camel_case_name)]));
                }
                ImplItem::Fn(method)
            } else {
                // Return non-method items unchanged
                item
            }
        })
        .collect();

    // Generate the output with wasm_bindgen applied to the impl block
    let output = quote! {
        #[wasm_bindgen]
        #input
    };

    output.into()
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
