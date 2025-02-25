use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Converts a function with "_original" suffix into a WASM-compatible function.
///
/// This macro:
/// 1. Keeps the original function
/// 2. Creates a new function without the "_original" suffix
#[proc_macro_attribute]
pub fn wasm_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Get the function name and create a new name without "_original" suffix
    let original_fn = &input_fn.sig.ident;
    let original_fn_name = original_fn.to_string();

    if !original_fn_name.ends_with("_original") {
        return syn::Error::new_spanned(
            original_fn,
            "Function name must end with '_original' to use this macro",
        )
        .to_compile_error()
        .into();
    }

    // Create the new name by removing "_original"
    let new_fn_name = original_fn_name.trim_end_matches("_original").to_string();

    // Convert snake_case to camelCase for JavaScript
    let js_name = new_fn_name
        .split('_')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_string()
            } else {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        })
        .collect::<String>();

    // Create the new function signature
    let mut new_fn_sig = input_fn.sig.clone();
    // Change the function name
    new_fn_sig.ident = syn::Ident::new(&new_fn_name, original_fn.span());

    let attrs = &input_fn.attrs;

    // Generate the output with both the original function and the new function with wasm_bindgen attribute
    let output = quote! {
        // Keep the original function
        #(#attrs)*
        #input_fn

        // Create the new function with wasm_bindgen attribute
        #[wasm_bindgen(js_name = #js_name)]
        pub #new_fn_sig {
            #original_fn()
        }
    };

    // Convert to string for debugging
    let output_str = output.to_string();

    // Print the generated code during compilation
    println!("GENERATED CODE:\n{}", output_str);

    output.into()
}
