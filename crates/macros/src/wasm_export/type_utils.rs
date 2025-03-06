use std::ops::Deref;
use syn::{ImplItemFn, Path, PathSegment, ReturnType, Type, TypePath};

/// Try to extract the inner type from a Result<T, E> type, returning None if not a Result
pub fn try_extract_result_inner_type(method: &ImplItemFn) -> Option<&Type> {
    if let ReturnType::Type(_, return_type) = &method.sig.output {
        if let Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) = return_type.deref()
        {
            if let Some(PathSegment {
                ident, arguments, ..
            }) = segments.first()
            {
                if *ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(args) = arguments {
                        if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                            return Some(t);
                        }
                    }
                }
            }
        }
    }
    None
}
