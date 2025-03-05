use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Creates a function call expression based on whether it's an instance or static method
pub fn create_new_function_call(
    fn_name: &Ident,
    has_self_receiver: bool,
    args: &[TokenStream],
) -> TokenStream {
    if has_self_receiver {
        // Instance method call
        quote! { self.#fn_name(#(#args),*) }
    } else {
        // Static method call
        quote! { Self::#fn_name(#(#args),*) }
    }
}
