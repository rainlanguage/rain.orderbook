use syn::{Path, PathSegment, Type, TypePath};

/// Try to extract the inner type from a Result<T, E> type, returning None if not a Result
pub fn try_extract_result_inner_type(return_type: &Type) -> Option<&Type> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = return_type
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
    None
}
