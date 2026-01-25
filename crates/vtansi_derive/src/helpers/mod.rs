//! Helper functions and utilities for derive macros.

pub mod builders;
pub mod debug;
pub mod default_variant;
pub mod doc_spans;
pub mod field_props;
pub mod metadata;
pub mod repr_type;
pub mod struct_params;
pub mod type_analysis;
pub mod type_props;
pub mod variant_props;

pub use self::builders::WriteOperationBuilder;
pub use self::debug::debug_print_generated;
pub use self::default_variant::{DefaultVariant, find_default_variant};
pub use self::doc_spans::generate_doc_imports;
pub use self::field_props::HasFieldProperties;
pub use self::metadata::ControlDirection;
pub use self::struct_params::{
    FieldInfo, StructParamInfo, extract_struct_param_info,
};
pub use self::type_analysis::extract_vec_inner_type;
pub use self::type_props::{ControlProperties, HasTypeProperties};

use proc_macro2::Span;
use quote::ToTokens;
use syn::{self, DeriveInput, GenericParam, Lifetime};

/// Return an error indicating that the macro can only be used on enums.
pub fn non_enum_error() -> syn::Error {
    syn::Error::new(Span::call_site(), "This macro only supports enums.")
}

/// Return an error indicating that the macro can only be used on structs.
pub fn non_struct_error() -> syn::Error {
    syn::Error::new(
        Span::call_site(),
        "This macro only supports structs with named fields.",
    )
}

/// Return an error for duplicate occurrences of an attribute.
///
/// Create an error message indicating that an attribute was specified
/// multiple times, with the second occurrence as the primary error and the
/// first occurrence as a note.
pub fn occurrence_error<T: ToTokens>(fst: T, snd: T, attr: &str) -> syn::Error {
    let mut e = syn::Error::new_spanned(
        snd,
        format!("Found multiple occurrences of vtansi({attr})"),
    );
    e.combine(syn::Error::new_spanned(fst, "first one here"));
    e
}

/// Return an error indicating that a required attribute is missing.
///
/// Create an error message with an example of how to specify the attribute.
pub fn error_required_attr(
    span: proc_macro2::Span,
    attr_name: &str,
    example: &str,
) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "{attr_name} attribute is required; add {attr_name} = {example}"
        ),
    )
}

/// Extract the inner type `T` from `Option<T>`.
///
/// Returns `None` if the type is not an `Option`.
#[allow(clippy::collapsible_if)]
pub fn extract_option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_some() {
            return None;
        }

        let path = &type_path.path;

        // Check for Option with generic argument
        let last_segment = path.segments.last()?;

        // Check if the last segment is "Option"
        if last_segment.ident != "Option" {
            return None;
        }

        // Check that the path is either just "Option" or "std/core::option::Option"
        let is_valid_path = if path.segments.len() == 1 {
            true
        } else if path.segments.len() == 3 {
            let segs: Vec<_> =
                path.segments.iter().map(|s| s.ident.to_string()).collect();
            (segs[0] == "std" || segs[0] == "core") && segs[1] == "option"
        } else {
            false
        };

        if !is_valid_path {
            return None;
        }

        // Extract the inner type from the angle brackets
        if let syn::PathArguments::AngleBracketed(args) =
            &last_segment.arguments
        {
            if args.args.len() == 1 {
                if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                    return Some(inner_ty);
                }
            }
        }
    }

    None
}

pub fn get_primary_lifetime(ast: &DeriveInput) -> Option<Lifetime> {
    for gp in ast.generics.params.iter() {
        if let GenericParam::Lifetime(lp) = gp {
            return Some(lp.lifetime.clone());
        }
    }

    None
}

/// Convert 0 -> "a", 1 -> "b", ..., 25 -> "z", 26 -> "ba", 27 -> "bb", ...
fn alpha_name_from_index(mut n: usize) -> String {
    let mut buf = Vec::new();
    loop {
        buf.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
        if n == 0 {
            break;
        }
    }
    buf.iter().rev().collect()
}

fn pick_alpha_lifetime(
    existing: &std::collections::HashSet<String>,
) -> Lifetime {
    // avoid special lifetimes
    let reserved = |s: &str| s == "static" || s == "_";

    let mut i = 0usize;
    loop {
        let candidate = alpha_name_from_index(i);
        if !existing.contains(&candidate) && !reserved(&candidate) {
            return Lifetime::new(&format!("'{candidate}"), Span::call_site());
        }
        i += 1;
    }
}

pub fn insert_lifetime(generics: &mut syn::Generics) -> Lifetime {
    let existing: std::collections::HashSet<_> = generics
        .lifetimes()
        .map(|lt| lt.lifetime.ident.to_string())
        .collect();

    let new_lt = pick_alpha_lifetime(&existing);

    generics.params.insert(
        0,
        syn::GenericParam::Lifetime(syn::LifetimeParam::new(new_lt.clone())),
    );

    new_lt
}
